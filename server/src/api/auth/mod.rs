use log::info;
use rocket::response;
use rocket::{http, Route, State};
use rocket_contrib::json;
use serde::Deserialize;

pub mod admin_token;
pub mod player_token;
pub mod session_id;
pub mod token;

pub use admin_token::AdminAuthToken;
pub use player_token::PlayerAuthToken;
pub use session_id::SessionID;
pub use token::BasicAuthToken;

pub(crate) static DEV_SECRET: &str = "dev-secret";

// all mounts go to /api/v*/ base
pub fn get_auth_api_routes() -> Vec<Route> {
    routes![get_status, connect_client, connect_admin]
}

/// Auth token system: every user stores a single token
/// Common fields: BasicAuthToken
/// - exp
/// - auth_level
///
/// PlayerAuthToken (auth_level="player"): extends BasicAuthToken with
/// - user_id
/// - session_id
/// - user_name
/// - role
///
/// ControlAuthToken (auth_level="control"): extends BasicAuthToken with
/// nothing? maybe add different levels of control
///

#[get("/status?<token>")]
fn get_status(token: Option<token::BasicAuthToken>) -> String {
    // TODO
    "unimplemented".into()
}

use crate::database::Database;
use crate::notify::{Notification, Notifier};
use serde::export::TryFrom;
use std::net::SocketAddr;

#[derive(Deserialize)]
struct ConnectData {
    username: String,
    session_id: String,
}

/// validates user connection request and if session exists and user is allowed to join, send jwt
#[post("/connect/client", data = "<conn_data>")]
fn connect_client(
    addr: SocketAddr,
    conn_data: json::Json<ConnectData>,
    db: State<Database>,
    notifier: State<Notifier>,
) -> response::status::Custom<String> {
    let conn_data = conn_data.into_inner();

    info!(
        "new connect request from {} as {} to session {}",
        addr, &conn_data.username, &conn_data.session_id
    );

    //TODO validate
    // validate session_id
    let sid: SessionID = match SessionID::try_from(conn_data.session_id.as_str()) {
        Ok(sid) => sid,
        Err(e) => return response::status::Custom(http::Status::BadRequest, e.to_string()),
    };

    match Database::maybe_add_player(db.get_locked_conn(), &conn_data.username, &sid) {
        Ok(uid) => {
            let jwt = PlayerAuthToken::get_jwt(uid, sid, conn_data.username, "".to_string());

            // tell others that new player has connected
            notifier.send(Notification::UpdatePlayerList(sid));

            response::status::Custom(http::Status::Ok, jwt)
        }
        Err(emsg) => return response::status::Custom(http::Status::BadRequest, emsg),
    }
}

#[derive(Deserialize)]
struct ConnectAdminData {
    password: String,
}

/// validates user connection request and if session exists and user is allowed to join, send jwt
#[post("/connect/ctrl", data = "<conn_data>")]
fn connect_admin(
    addr: SocketAddr,
    conn_data: json::Json<ConnectAdminData>,
) -> response::status::Custom<String> {
    let conn_data = conn_data.into_inner();

    info!(
        "new admin connect request from {} with pwd {}",
        addr, &conn_data.password
    );

    let jwt = AdminAuthToken::get_jwt();
    response::status::Custom(http::Status::Ok, jwt)
}
