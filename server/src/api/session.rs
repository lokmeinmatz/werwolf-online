use crate::api::auth::token::AuthToken;
use rocket_contrib::json::Json;
use rocket::{Route, State};
use crate::PlayerData;
use crate::database::Database;
use crate::api::auth::SessionID;
use crate::api::auth::admin_token::AdminToken;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist, get_all_sessions]
}


#[get("/")]
fn get_all_sessions(ctrl: AdminToken, db: State<Database>) -> Json<Vec<String>> {
    Json(Database::get_all_sessions(db.get_locked_conn(), |r| {
        r.get()
    }))
}


#[get("/<sid>/playerlist", format = "json")]
fn get_playerlist(sid: SessionID, auth: AuthToken, db: State<Database>) ->
                                                                        Option<Json<Vec<PlayerData>>> {
    if &sid != auth.session_id() { return None }
    Some(Json(Database::get_players(db.get_locked_conn(), auth.session_id())))
}
