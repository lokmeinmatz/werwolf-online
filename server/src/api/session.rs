use crate::api::auth::token::AuthToken;
use rocket_contrib::json::Json;
use rocket::{Route, State};
use crate::PlayerData;
use crate::database::Database;

pub fn get_session_api_routes() -> Vec<Route> {
    routes![get_playerlist]
}





#[get("/playerlist", format = "json")]
fn get_playerlist(auth: AuthToken, db: State<Database>) -> Json<Vec<PlayerData>> {
    Json(Database::get_players(db.get_locked_conn(), auth.session_id()))
}
