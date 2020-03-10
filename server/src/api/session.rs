use crate::api::auth::{SessionID, PlayerAuthToken};
use crate::api::auth::{AdminAuthToken, BasicAuthToken};
use crate::database::Database;
use crate::PlayerData;
use rocket::{Route, State};
use rocket_contrib::json::Json;
use std::convert::TryFrom;
use serde::Serialize;
use std::ops::Add;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist, get_all_sessions]
}

#[derive(Serialize)]
struct BasicSessionInfo {
    id: String,
    player_count: u32,
    active: bool,
    created: u64
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
            active: r.get(2)?
        })
    });

    for s in &mut sessions {
        // get players in session
        s.player_count = Database::get_players(
            &mut locked,
            &SessionID::try_from(s.id.as_str()).unwrap()).len() as u32;
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
    Some(Json(Database::get_players(
        &mut db.get_locked_conn(),
        &sid,
    )))
}
