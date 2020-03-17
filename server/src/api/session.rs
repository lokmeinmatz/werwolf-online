use crate::api::auth::{AdminAuthToken, BasicAuthToken};
use crate::api::auth::{PlayerAuthToken, SessionID};
use crate::api::net_types::{BasicSessionInfo, PlayerData};
use crate::database::Database;
use crate::SessionData;
use rocket::{Route, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use std::convert::TryFrom;
use std::ops::Add;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist, get_all_sessions, get_session_info]
}

impl From<SessionData> for BasicSessionInfo {
    fn from(sd: SessionData) -> Self {
        BasicSessionInfo {
            id: sd.id.to_string(),
            player_count: 0,
            active: sd.active,
            created: sd
                .created
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[get("/")]
fn get_all_sessions(ctrl: AdminAuthToken, db: State<Database>) -> Json<Vec<BasicSessionInfo>> {
    let mut locked = db.get_locked_conn();
    let mut sessions = Database::get_all_sessions(&mut locked, |r| {
        let secs_unix: i64 = r.get(1)?;
        Ok(BasicSessionInfo {
            id: r.get(0)?,
            player_count: 0,
            created: secs_unix as u64,
            active: r.get(2)?,
        })
    });

    for s in &mut sessions {
        // get players in session
        s.player_count =
            Database::get_players(&mut locked, &SessionID::try_from(s.id.as_str()).unwrap()).len()
                as u32;
    }

    Json(sessions)
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
    Some(Json(Database::get_players(&mut db.get_locked_conn(), &sid)))
}

#[get("/<sid>", format = "json")]
fn get_session_info(
    sid: SessionID,
    auth: BasicAuthToken,
    db: State<Database>,
) -> Option<Json<BasicSessionInfo>> {
    match Database::get_session_data(&mut db.get_locked_conn(), &sid) {
        Some(sd) => Some(Json(sd.into())),
        None => None,
    }
}
