use rocket::response;
use rocket::{Rocket, State};

use log::{error, info, warn};

use crate::api::auth::SessionID;
use crate::database::Database;
use crate::api::auth::admin_token::AdminToken;

pub fn mount_controller_api(mut rocket: Rocket) -> Rocket {
    rocket.mount("/ctrl", routes![get_login_page, get_overview_page, get_overview_page_err,
    get_controller_page])
}

#[get("/")]
pub fn get_login_page() -> response::NamedFile {
    response::NamedFile::open("../webapp/dist/controller_login.html").unwrap()
}

#[get("/overview")]
pub fn get_overview_page(
    _auth: AdminToken
) -> response::NamedFile {
    response::NamedFile::open("../webapp/dist/controller_overview.html").unwrap()
}

#[get("/overview", rank = 2)]
pub fn get_overview_page_err(
) -> response::Redirect {
    response::Redirect::to("/ctrl/?error=NoToken")
}

#[get("/<sid>", rank = 3)]
pub fn get_controller_page(
    _auth: AdminToken,
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
