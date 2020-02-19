use rocket::{Route, State};
use rocket::http::uri::Origin;
use crate::notify::{Notifier, Notification};
use crate::api::auth::SessionID;
use log::{error};
use std::convert::TryFrom;
use rocket_contrib::json::Json;
use serde::Serialize;
use std::sync::{Arc, atomic::AtomicI64};
use crate::database::Database;
use std::sync::atomic::Ordering;
use std::time::Duration;

pub mod auth;
pub mod session;

fn extend_with_base(base: Origin, mut routes: Vec<Route>) -> Vec<Route> {
    for mut route in &mut routes {
        let inner_uri = route.uri.clone();
        route.set_uri(base.clone(), inner_uri);
    }

    routes
}

/// Gets api routes <...> so that /api/v1/<...> should get exposed
/// So gets mounted to /api/v1
pub fn get_current_api_routes() -> Vec<Route> {
    let mut routes: Vec<Route> = routes![stats];

    // mount auth
    let auth_base = Origin::parse("/auth").unwrap();
    routes.append(&mut extend_with_base(auth_base, auth::get_auth_api_routes()));


    // mount session
    let session_base = Origin::parse("/session").unwrap();
    routes.append(&mut extend_with_base(session_base, session::get_session_api_routes()));


    routes
}

#[derive(Serialize)]
struct Stats {
    ws_connected: u32,
    sessions_active: u32,
    unique_users: u64
}

#[get("/stats")]
fn stats(notifier: State<Notifier>, db: State<Database>) -> Json<Stats> {
    // get ws connected
    let mut shared_ws_connected = Arc::new(AtomicI64::new(-1));
    notifier.send(Notification::UpdateConnectionsAlive(Arc::clone(&shared_ws_connected)));

    let conn = db.get_locked_conn();
    let sessions_active = Database::get_sessions_active(conn);
    let mut ws_connected: i64= -1;
    for _ in 0..8 {
        // try if was updated
        ws_connected =  shared_ws_connected.load(Ordering::Relaxed);
        if ws_connected >= 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    if ws_connected < 0 {
        ws_connected = 0;
        error!("Notifier didn't respond");
    }

    Json(Stats{
        ws_connected: ws_connected as u32,
        sessions_active,
        unique_users: 0
    })
}