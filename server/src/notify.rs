use crate::api::auth::SessionID;
use log::{error, info, warn};
use std::cell::Cell;
use std::convert::TryFrom;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{self, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tungstenite::handshake::server::ErrorResponse;
use tungstenite::protocol::WebSocket;
use tungstenite::Message;

pub enum Notification {
    UpdatePlayerList(SessionID),
    CustomToPlayer(u64, String),
    CustomToSession(SessionID, String),
    UpdateConnectionsAlive(Arc<atomic::AtomicI64>),
}

pub struct Notifier(Mutex<mpsc::Sender<Notification>>);

impl Notifier {
    pub fn send(&self, msg: Notification) {
        match self.0.lock().unwrap().send(msg) {
            Err(_) => {
                error!("Failed to send Notification");
            }
            _ => {}
        }
    }
}

pub fn start(addr: SocketAddr) -> std::io::Result<Notifier> {
    info!(
        target: WS_LOG_TARGET,
        "Initializing WebSocketHandler on addr {:?}", addr
    );

    let (sender, receiver) = mpsc::channel();

    std::thread::Builder::new()
        .name("WebsocketWorker".into())
        .spawn(move || {
            // create server
            let server = TcpListener::bind(addr).unwrap();
            server.set_nonblocking(true);

            let mut handler = WebsocketHandler::new(receiver);

            while !crate::SHOULD_TERMINATE.load(Ordering::Relaxed) {
                // check for new incoming data
                while let Ok((new_stream, client_addr)) = server.accept() {
                    // hacky to get session id from request

                    // if the inner option contains Some(sid), it was from and player
                    // else if the inner Option is None, it came from and controller
                    let mut conn_data: Cell<Option<Option<SessionID>>> = Cell::new(None);

                    if let Ok(ws) = tungstenite::accept_hdr(
                        new_stream,
                        |req: &tungstenite::handshake::server::Request,
                         res: tungstenite::handshake::server::Response| {
                            // --- CALLBACK START ---
                            // validate ws-upgrade was called with valid token

                            let mut uri_path = req.uri().path();

                            if uri_path.len() < 4 {
                                return Err(ErrorResponse::new(Some("no valid token".to_owned())));
                            }
                            // remove / from uri
                            uri_path = uri_path
                                .chars()
                                .next()
                                .map(|c| &uri_path[c.len_utf8()..])
                                .unwrap();
                            info!("Parsing {}", uri_path);
                            // parse
                            use crate::api::auth::PlayerAuthToken;
                            match PlayerAuthToken::try_from(uri_path) {
                                Ok(at) => {
                                    info!(target: WS_LOG_TARGET, "got valid request: {:?}", &at);
                                    conn_data.set(Some(Some(at.session_id)));
                                    return Ok(res);
                                }
                                Err(_) => {
                                    error!(
                                        target: WS_LOG_TARGET,
                                        "could not parse user token, trying \
                            with admin"
                                    );
                                }
                            }

                            use crate::api::auth::AdminAuthToken;
                            match AdminAuthToken::try_from(uri_path) {
                                Ok(at) => {
                                    info!("Got valid admin request: {:?}", &at);
                                    conn_data.set(Some(None));
                                    return Ok(res);
                                }
                                Err(_) => error!(target: WS_LOG_TARGET, "No valid admin token"),
                            }

                            Err(ErrorResponse::new(Some("Invalid token".to_owned())))

                            // --- CALLBACK END ---
                        },
                    ) {
                        if let Some(inner) = conn_data.replace(None) {
                            info!(
                                target: WS_LOG_TARGET,
                                "New WS connection from {:?}", client_addr
                            );
                            let conn = match inner {
                                Some(sid) => WSConnection::Player(sid, ws),
                                None => WSConnection::Controller(ws),
                            };
                            handler.add_socket(conn);
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

enum WSConnection {
    Player(SessionID, WebSocket<TcpStream>),
    Controller(WebSocket<TcpStream>),
}

impl WSConnection {
    fn get_ws(&mut self) -> &mut WebSocket<TcpStream> {
        match self {
            WSConnection::Player(_, ws) => ws,
            WSConnection::Controller(ws) => ws,
        }
    }

    fn associated_w_sid(&self, sid: &SessionID) -> bool {
        match self {
            WSConnection::Controller(_) => true,
            WSConnection::Player(ref psid, _) => psid.eq(sid),
        }
    }
}

struct WebsocketHandler {
    message_queue: mpsc::Receiver<Notification>,
    connections: Vec<WSConnection>,
    dead_sockets: Vec<usize>,
}

impl WebsocketHandler {
    pub fn new(msg_queue: mpsc::Receiver<Notification>) -> Self {
        WebsocketHandler {
            message_queue: msg_queue,
            connections: Vec::new(),
            dead_sockets: Vec::new(),
        }
    }

    pub fn poll_messages(&mut self) {
        for (idx, ws) in self.connections.iter_mut().map(|e| e.get_ws()).enumerate() {
            if !(ws.can_read() && ws.can_write()) {
                self.dead_sockets.push(idx);
                info!("Ws cant read or write: {:?}", ws.get_ref().peer_addr());
                continue;
            }

            match ws.read_message() {
                Ok(Message::Close(_)) => {
                    info!("Close message from: {:?}", ws.get_ref().peer_addr());
                    self.dead_sockets.push(idx);
                }
                _ => {}
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

        warn!(
            "Called clean_up: May slow ws service down! Removing {} ws",
            self.dead_sockets.len()
        );

        self.dead_sockets.sort_unstable_by(|a, b| b.cmp(a));
        info!("{:?}", self.dead_sockets);
        for &d_s_idx in &self.dead_sockets {
            self.connections.remove(d_s_idx);
        }

        self.dead_sockets.clear();

        info!("Sucessfully cleaned up dead sockets :D");
    }

    pub fn handle_notifications(&mut self) {
        while let Ok(msg) = self.message_queue.try_recv() {
            match msg {
                Notification::UpdatePlayerList(sid) => {
                    for client in self
                        .connections
                        .iter_mut()
                        .filter(|e| e.associated_w_sid(&sid))
                    {
                        client
                            .get_ws()
                            .write_message(Message::Text("update.playerlist".to_owned()));
                    }
                }
                Notification::UpdateConnectionsAlive(res) => {
                    res.store(self.connections.len() as i64, Ordering::Relaxed);
                    info!(
                        target: WS_LOG_TARGET,
                        "Updated shared alive-counter to {}",
                        self.connections.len()
                    );
                }
                _ => {
                    warn!(
                        target: WS_LOG_TARGET,
                        "Unknown Notification-type: Maybe check if you \
                    implemented all variants?"
                    );
                }
            }
        }
    }

    pub fn add_socket(&mut self, conn: WSConnection) {
        self.connections.push(conn)
    }

    pub fn terminate(mut self) {
        info!(
            target: WS_LOG_TARGET,
            "Terminating: closing all ws-connections..."
        );
        for mut conn in self.connections {
            conn.get_ws().close(None);
        }
        info!(target: WS_LOG_TARGET, "Terminated!");
    }
}

static WS_LOG_TARGET: &'static str = "WebSocket";
