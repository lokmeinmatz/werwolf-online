use rocket::{Route, State};
use rocket::http::uri::Origin;
use crate::notify::{Notifier, Notification};
use crate::api::auth::SessionID;
use log::{error};
use std::convert::TryFrom;

pub mod auth;

/// Gets api routes <...> so that /api/v1/<...> should get exposed
/// So gets mounted to /api/v1
pub fn get_current_api_routes() -> Vec<Route> {
    let mut routes: Vec<Route> = routes![testpoint];

    // mount auth
    let auth_base = Origin::parse("/auth").unwrap();
    for mut route in auth::mount_auth_api() {
        let inner_uri = route.uri.clone();
        route.set_uri(auth_base.clone(), inner_uri);
        routes.push(route);
    }

    routes
}


#[get("/test/<cmd>/<sid>")]
fn testpoint(ntfy: State<Notifier>, cmd: String, sid: Option<String>) -> String {

    match (cmd.as_str(), sid) {
        ("update.playerlist", Some(ref sid)) => {
            match SessionID::try_from(sid.as_str()) {
                Ok(s) => {
                    ntfy.send(Notification::UpdatePlayerList(s));
                    format!("send UpdatePlayerList for sid {}", sid)
                },
                Err(_) => {
                    format!("invalid session id: {}", sid)
                }
            }
        },
        _ => {
            error!("request with unknown command: {}", cmd);
            format!("unknown command: {}", cmd)
        }
    }
}