use crate::api::auth::{SessionID, PlayerAuthToken};
use crate::api::auth::{AdminAuthToken, BasicAuthToken};
use crate::database::Database;
use crate::PlayerData;
use rocket::{Route, State};
use rocket_contrib::json::Json;
use std::convert::TryFrom;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist, get_all_sessions]
}

#[get("/")]
fn get_all_sessions(ctrl: AdminAuthToken, db: State<Database>) -> Json<Vec<String>> {
    Json(Database::get_all_sessions(db.get_locked_conn(), |r| {
        r.get(0)
    }))
}

#[get("/<sid>/playerlist", format = "json")]
fn get_playerlist(
    sid: SessionID,
    auth: BasicAuthToken,
    db: State<Database>,
) -> Option<Json<Vec<PlayerData>>> {
    if let Ok(player_auth) = PlayerAuthToken::try_from(auth) {
        if player_auth.session_id != sid {
            return None;
        }
    }
    Some(Json(Database::get_players(
        db.get_locked_conn(),
        &sid,
    )))
}
