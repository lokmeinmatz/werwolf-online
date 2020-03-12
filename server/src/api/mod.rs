use crate::database::Database;
use crate::notify::{Notification, Notifier};
use log::error;
use log::info;
use rocket::http::uri::Origin;
use rocket::{Rocket, Route, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicI64, Arc};
use std::time::Duration;

pub mod auth;
pub mod session;

/// Gets api routes <...> so that /api/v1/<...> should get exposed
/// So gets mounted to /api/v1
pub fn mount_current_api_routes(mut rocket: Rocket) -> Rocket {
    rocket
        .mount("/api/v1/", routes![stats])
        .mount("/api/v1/auth/", auth::get_auth_api_routes())
        .mount("/api/v1/sessions/", session::get_session_api_routes())
}

#[derive(Serialize)]
struct Stats {
    ws_connected: u32,
    sessions_active: u32,
    unique_users: u64,
}

#[get("/stats")]
fn stats(notifier: State<Notifier>, db: State<Database>) -> Json<Stats> {
    // get ws connected
    let shared_ws_connected = Arc::new(AtomicI64::new(-1));
    notifier.send(Notification::UpdateConnectionsAlive(Arc::clone(
        &shared_ws_connected,
    )));

    let conn = db.get_locked_conn();
    let sessions_active = Database::get_sessions_active(conn);
    let mut ws_connected: i64 = -1;
    for _ in 0..8 {
        // try if was updated
        ws_connected = shared_ws_connected.load(Ordering::Relaxed);
        if ws_connected >= 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    if ws_connected < 0 {
        ws_connected = 0;
        error!("Notifier didn't respond");
    }

    Json(Stats {
        ws_connected: ws_connected as u32,
        sessions_active,
        unique_users: 0,
    })
}
