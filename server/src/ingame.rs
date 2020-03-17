use rocket::response;
use rocket::{Rocket, State};

use log::{error, info, warn};

use crate::api::auth::PlayerAuthToken;
use crate::database::Database;

pub fn mount_ingame_pages(mut rocket: Rocket) -> Rocket {
    rocket.mount("/game", routes![get_game_page, get_game_page_noauth])
}

#[get("/")]
pub fn get_game_page(
    auth: PlayerAuthToken,
    db: State<Database>,
) -> Result<response::NamedFile, response::Redirect> {
    info!("New page request to session {}", auth.session_id);

    if Database::get_session_data(&mut db.get_locked_conn(), &auth.session_id).is_none() {
        warn!("Invalid session requested");
        Err(response::Redirect::to("/?error=InvalidSessionID"))
    //Err(response::Redirect::to("/"))
    } else {
        let res = response::NamedFile::open("../webapp/dist/ingame.html")
            .map_err(|e| error!("Failed to host file: {:?}", e))
            .unwrap();
        //Cache-Control: max-age=3600, must-revalidate
        Ok(res)
    }
}

#[get("/", rank = 2)]
pub fn get_game_page_noauth() -> response::Redirect {
    warn!("New page request to game/, but no authtoken provided");

    response::Redirect::to("/?error=NoToken")
}
