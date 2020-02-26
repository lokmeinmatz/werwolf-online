use crate::api::auth::token::AuthToken;
use rocket_contrib::json::Json;
use rocket::{Route, State};
use crate::PlayerData;
use crate::database::Database;
use crate::api::auth::SessionID;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist]
}





#[get("/<sid>/playerlist", format = "json")]
fn get_playerlist(sid: SessionID, auth: AuthToken, db: State<Database>) ->
                                                                        Option<Json<Vec<PlayerData>>> {
    if &sid != auth.session_id() { return None }
    Some(Json(Database::get_players(db.get_locked_conn(), auth.session_id())))
}
