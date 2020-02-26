use rocket::{http, Route, State};
use rocket::{request, response};
use serde::{Deserialize};
use log::{info};
use rocket_contrib::json;

pub mod token;
pub mod admin_token;

pub(crate) static DEV_SECRET: &str = "dev-secret";

// all mounts go to /api/v*/ base
pub fn get_auth_api_routes() -> Vec<Route> {
    routes![get_status, connect_client, connect_admin]
}




const SID_LENGTH: usize = 8;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SessionID([u8; SID_LENGTH]);

impl SessionID {

    // when the SessionID is created, the content is allready validated
    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.0)
        }
    }
}

impl TryFrom<&str> for SessionID {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != SID_LENGTH {
            return Err("Invalid id-length");
        }
        if !value.chars().all(|c| (c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_uppercase())) && c.len_utf8() == 1) {
            return Err("session-id must consist of digits and uppercase ascii letters");
        }
        let mut buf = ['-' as u8; SID_LENGTH];
        for (i, c) in value.chars().enumerate() {
            buf[i] = c as u8;
        }
        Ok(SessionID(buf))
    }
}

/*impl Serialize for SessionID {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_str(self.as_str())
    }
}

impl Deserialize for SessionID {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        deserializer.deserialize_str()
    }
}*/

impl std::fmt::Display for SessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "SID:{}", self.as_str())
    }
}

impl std::fmt::Debug for SessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "SID:{}", self.as_str())
    }
}

impl<'q> request::FromParam<'q> for SessionID {
    type Error = &'static str;

    fn from_param(param: &'q http::RawStr) -> Result<Self, Self::Error> {
        if param.len() != SID_LENGTH {
            return Err("sessionID with wrong length");
        }
        let sid: String = param.url_decode_lossy();

        info!("raw sid: {}", sid);

        if !sid.chars().all(|c| c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_uppercase())) {
            return Err("sessionID must by uppercase letters or digits (radix 10)");
        }

        let mut sid_chars = ['-' as u8; SID_LENGTH];
        for (i, c) in sid.chars().enumerate() {
            sid_chars[i] = c as u8;
        }
        Ok(SessionID(sid_chars))
    }
}

#[get("/status?<token>")]
fn get_status(token: Option<token::AuthToken>) -> String {
    let s: String = match token {
        Some(t) => {
            format!("username: {} | current session: {}", t.username(), t.session_id())
        },
        None => {
            "No token provided".into()
        }
    };

    s
}



use std::net::SocketAddr;
use serde::export::{Formatter, TryFrom};
use serde::export::fmt::Error;
use crate::database::Database;
use crate::notify::{Notifier, Notification};

#[derive(Deserialize)]
struct ConnectData {
    username: String,
    session_id: String
}



/// validates user connection request and if session exists and user is allowed to join, send jwt
#[post("/connect/client", data = "<conn_data>")]
fn connect_client(addr: SocketAddr, conn_data: json::Json<ConnectData>, db: State<Database>,
                notifier: State<Notifier>)
                -> response::status::Custom<String> {

    let conn_data = conn_data.into_inner();

    info!("new connect request from {} as {} to session {}",
          addr,
          &conn_data.username,
          &conn_data.session_id
    );

    //TODO validate
    // validate session_id
    let sid: SessionID = match SessionID::try_from(conn_data.session_id.as_str()) {
        Ok(sid) => sid,
        Err(e) => return response::status::Custom(http::Status::BadRequest, e.to_string())
    };

    match Database::maybe_add_player(db.get_locked_conn(), &conn_data.username, &sid) {
        Ok(uid) => {
            match token::gen_jwt(
                conn_data.username,
                uid,
                &sid) {
                Ok(jwt) => {

                    // tell others that new player has connected
                    notifier.send(Notification::UpdatePlayerList(sid));

                    response::status::Custom(http::Status::Ok, jwt)

                },
                Err(_) => response::status::Custom(http::Status::InternalServerError, "Failed to \
                generate token".into())
            }
        },
        Err(emsg) => {
            return response::status::Custom(http::Status::BadRequest, emsg);
        }
    }

}


#[derive(Deserialize)]
struct ConnectAdminData {
    password: String
}


/// validates user connection request and if session exists and user is allowed to join, send jwt
#[post("/connect/ctrl", data = "<conn_data>")]
fn connect_admin(addr: SocketAddr, conn_data: json::Json<ConnectAdminData>)
                  -> response::status::Custom<String> {

    let conn_data = conn_data.into_inner();

    info!("new admin connect request from {} with pwd {}",
          addr,
          &conn_data.password
    );

    match admin_token::gen_admin_jwt() {
        Ok(jwt) => {
            response::status::Custom(http::Status::Ok, jwt)
        },
        Err(_) => response::status::Custom(http::Status::InternalServerError, "Failed to generate\
         token".into())
    }
}