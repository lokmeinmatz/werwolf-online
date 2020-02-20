use crate::api::auth::SessionID;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{mpsc, Mutex, Arc};
use tungstenite::protocol::WebSocket;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::convert::TryFrom;
use std::time::Duration;
use std::sync::atomic::{self, Ordering};
use tungstenite::handshake::server::ErrorResponse;
use std::rc::Rc;
use tungstenite::Message;
use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use by_address::ByAddress;


pub enum Notification {
    UpdatePlayerList(SessionID),
    CustomToPlayer(String, String),
    CustomToSession(SessionID, String),
    UpdateConnectionsAlive(Arc<atomic::AtomicI64>)
}

pub struct Notifier(Mutex<mpsc::Sender<Notification>>);


impl Notifier {
    pub fn send(&self, msg: Notification) {
        match self.0.lock().unwrap().send(msg) {
            Err(_) => { error!("Failed to send Notification"); }
            _ => {}
        }
    }
}

pub fn start(addr: SocketAddr) -> std::io::Result<Notifier> {

    info!(target: WS_LOG_TARGET, "Initializing WebSocketHandler on addr {:?}", addr);

    let (sender, receiver) = mpsc::channel();

    std::thread::Builder::new().name("WebsocketWorker".into()).spawn(move || {
        // create server
        let server = TcpListener::bind(addr).unwrap();
        server.set_nonblocking(true);

        let mut handler = WebsocketHandler::new(receiver);

        while !crate::SHOULD_TERMINATE.load(Ordering::Relaxed) {

            // check for new incoming data
            while let Ok((new_stream, client_addr)) = server.accept() {

                // hacky to get session id from request
                let mut user_data: Cell<Option<(String, SessionID)>> = Cell::new(None);

                if let Ok(ws) = tungstenite::accept_hdr(new_stream, |req:
                    &tungstenite::handshake::server::Request, res:
                    tungstenite::handshake::server::Response| {

                    // --- CALLBACK START ---
                    // validate ws-upgrade was called with valid token

                    let mut uri_path = req.uri().path();

                    if uri_path.len() < 4 {
                        return Err(ErrorResponse::new(Some("no valid token".to_owned())));
                    }
                    // remove / from uri
                    uri_path = uri_path.chars().next().map(|c| &uri_path[c.len_utf8()..]).unwrap();

                    // parse
                    use crate::api::auth::token::AuthToken;
                    let atk = AuthToken::try_from(uri_path);
                    match atk {
                        Ok(at) => {
                            info!(target: WS_LOG_TARGET, "got valid request: {:?}", &at);
                            user_data.set(Some(at.inner()));
                            Ok(res)
                        },
                        Err(_) => {
                            error!(target: WS_LOG_TARGET, "could not parse token");
                            Err(ErrorResponse::new(Some("Invalid token".to_owned())))
                        }
                    }

                    // --- CALLBACK END ---
                }
                ) {
                    if let Some((username, s_id)) = user_data.replace(None) {
                        info!(target: WS_LOG_TARGET, "New WS connection from {:?}", client_addr);
                        handler.add_socket(ws, s_id, username);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(1));

            // process notifications to send
            handler.poll_messages();
            handler.handle_notifications();

            if handler.needs_cleanup() {
                handler.clean_up()
            }

        }


        // cleanup
        handler.terminate();
    });

    Ok(Notifier(Mutex::new(sender)))

}

type SharedWS = Rc<RefCell<WebSocket<TcpStream>>>;


struct WebsocketHandler {
    message_queue: mpsc::Receiver<Notification>,
    clients_by_sid: HashMap<SessionID, Vec<SharedWS>>,
    dead_sockets: HashSet<ByAddress<SharedWS>>
}



impl WebsocketHandler {

    pub fn new(msg_queue: mpsc::Receiver<Notification>) -> Self {
        WebsocketHandler {
            message_queue: msg_queue,
            clients_by_sid: HashMap::new(),
            dead_sockets: HashSet::new()
        }
    }

    pub fn poll_messages(&mut self) {
        for (_, bucket) in &mut self.clients_by_sid {
            for ws in bucket {
                let mut ws_ref: RefMut<WebSocket<TcpStream>> = (**ws).borrow_mut();

                if !(ws_ref.can_read() && ws_ref.can_write()) {
                    self.dead_sockets.insert(ByAddress(ws.clone()));
                    info!("Ws cant read or write: {:?}", ws_ref.get_ref().peer_addr());
                    continue;
                }

                match ws_ref.read_message() {
                    Ok(Message::Close(_)) => {
                        info!("Close message from: {:?}", ws_ref.get_ref().peer_addr());
                        self.dead_sockets.insert(ByAddress(ws.clone()));
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn needs_cleanup(&self) -> bool {
        self.dead_sockets.len() > 0
    }

    pub fn clean_up(&mut self) {
        if self.dead_sockets.len() == 0 {
            info!("No sockets to remove");
            return;
        }

        warn!("Called clean_up: May slow ws service down! Removing {} ws",
              self.dead_sockets.len());

        let mut dead_sockets = std::mem::replace(&mut self.dead_sockets, HashSet::new());

        for (sid, session_bucket) in &mut self.clients_by_sid {

            session_bucket.drain_filter(|ws| {
                // TODO optimize so we dont have to colne the rc
                dead_sockets.contains(&ByAddress(ws.clone()))
            });
        }

        info!("Sucessfully cleaned up dead socekts :D");


    }

    pub fn handle_notifications(&mut self) {
        while let Ok(msg) = self.message_queue.try_recv() {
            match msg {
                Notification::UpdatePlayerList(sid) => {
                    if let Some(clients) = self.clients_by_sid.get(&sid) {
                        for client_refcell in clients {
                            let mut client = (**client_refcell).borrow_mut();
                            if client.can_write() {
                                client.write_message(Message::Text("update.playerlist".to_owned()));
                            }
                        }
                    }
                },
                Notification::UpdateConnectionsAlive(res) => {

                    let mut alive = 0;

                    for (_, session_bucket) in &mut self.clients_by_sid {
                        alive += session_bucket.len();
                    }

                    res.store(alive as i64, Ordering::Relaxed);
                    info!("Updated shared alive-counter to {}",
                          alive);
                }
                _ => {
                    warn!(target: WS_LOG_TARGET, "Unknown Notification-type: Maybe check if you \
                    implemented all variants?");
                }
            }
        }

    }

    pub fn add_socket(&mut self, socket: WebSocket<TcpStream>, session: SessionID, username:
    String) {
        let rcd_ws: SharedWS = Rc::new(RefCell::new(socket));
        self.clients_by_sid.entry(session)
            .or_insert_with(|| Vec::with_capacity(1)).push(rcd_ws);
    }

    pub fn terminate(mut self) {
        info!(target: WS_LOG_TARGET, "Terminating: closing all ws-connections...");
        for bucket in self.clients_by_sid.values_mut() {
            for client in bucket {
                (**client).borrow_mut().close(None);
            }
        }
        info!(target: WS_LOG_TARGET, "Terminated!");
    }
}

static WS_LOG_TARGET : &'static str = "WebSocket";
