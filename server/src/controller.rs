use rocket::response;
use rocket::{Rocket, State};

use log::{error, info, warn};

use crate::api::auth::token::AuthToken;
use crate::api::auth::SessionID;
use crate::database::Database;

pub fn mount_controller_api(mut rocket: Rocket) -> Rocket {
    rocket.mount("/ctrl", routes![get_login_page, get_controller_page])
}

#[get("/")]
pub fn get_login_page() -> response::NamedFile {
    response::NamedFile::open("../webapp/dist/controller_login.html").unwrap()
}

#[get("/<sid>")]
pub fn get_controller_page(
    auth: AuthToken,
    sid: SessionID,
    db: State<Database>,
) -> Result<response::NamedFile, response::Redirect> {
    match Database::get_session_data(db.get_locked_conn(), &sid) {
        Some(_) => Ok(response::NamedFile::open("../webapp/dist/controller_login.html").unwrap()),
        None => {
            warn!("Admin requested invalid session");
            Err(response::Redirect::to("ctrl/?error=InvalidSessionID"))
        },
    }


}
