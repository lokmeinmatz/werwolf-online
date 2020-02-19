use rocket::Rocket;
use rocket::response;

use log::{info, warn, error};

use crate::api;

pub fn mount_ingame_api(mut rocket: Rocket) -> Rocket {
    rocket.mount("/game", routes![get_game_page])
}


#[get("/<sid>")]
pub fn get_game_page(sid: api::auth::SessionID) -> Result<response::NamedFile, response::Redirect> {

    info!("New page request to session {}", sid);

    if sid.as_str() != "1234ASDF" {
        warn!("Invalid session requested");
        Err(response::Redirect::to("/?error=InvalidSessionID"))
        //Err(response::Redirect::to("/"))
    }
    else {
        Ok(response::NamedFile::open("../webapp/dist/ingame.html").map_err(|e| error!("Failed to host file: {:?}", e)).unwrap())
    }

}