use super::SessionID;
use jsonwebtoken as jwt;
use jsonwebtoken::{TokenData, Validation};
use rocket::http::RawStr;
use rocket::request::{Outcome};
use rocket::{request, Request};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use log::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct BasicAuthClaims {
    exp: u64,
    pub(crate) sub: String,
    pub(crate) session_id: String,
    pub(crate) user_id: u32
}

#[derive(Debug)]
pub enum AuthClaimError {
    NoAuthCookie,
    Expired,
    Blocked,
}

static DEV_SECRET: &str = "dev-secret";

pub fn gen_jwt(username: String, user_id: u32, sid: &SessionID) -> Result<String, ()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let norole = BasicAuthClaims {
        // token expires after 4h
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ())?
            .as_secs()
            + 3600 * 4,
        sub: username,
        user_id,
        session_id: sid.as_str().to_owned(),
    };

    jwt::encode(&jwt::Header::default(), &norole, DEV_SECRET.as_bytes()).map_err(|_| ())
}

#[derive(Debug)]
pub struct AuthToken {
    exp: u64,
    username: String,
    user_id: u32,
    session_id: SessionID,
}

impl AuthToken {
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn session_id(&self) -> &SessionID {
        &self.session_id
    }

    /// Returns (username, session_id) as owned values.
    pub fn inner(self) -> (String, SessionID) {
        (self.username, self.session_id)
    }
}

impl std::convert::TryFrom<BasicAuthClaims> for AuthToken {
    type Error = ();

    fn try_from(value: BasicAuthClaims) -> Result<Self, Self::Error> {
        let sid = SessionID::try_from(value.session_id.as_str()).map_err(|_| ())?;

        Ok(AuthToken {
            exp: value.exp,
            session_id: sid,
            user_id: value.user_id,
            username: value.sub,
        })
    }
}

impl TryFrom<&str> for AuthToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // try to decode jwt
        let tdata: TokenData<BasicAuthClaims> =
            jwt::decode(value, DEV_SECRET.as_bytes(), &Validation::default()).map_err(|e| {
                warn!("Deconding of jwt failed: {}", e.to_string());
                ()
            })?;
        tdata.claims.try_into().map_err(|_| ())
    }
}

impl<'r> request::FromFormValue<'r> for AuthToken {
    type Error = &'static str;

    fn from_form_value(form_value: &'r RawStr) -> Result<Self, Self::Error> {
        // try to decode jwt
        let tdata: TokenData<BasicAuthClaims> = jwt::decode(
            form_value.as_str(),
            DEV_SECRET.as_bytes(),
            &Validation::default(),
        )
        .map_err(|_| "No valid JWT")?;

        tdata.claims.try_into().map_err(|_| "invalid claims")
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for AuthToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(hdr) => {
                let splitted: Vec<&str> = hdr.split(" ").collect();
                info!("got Authorization header: {:?}", splitted);
                if splitted.len() == 2 && splitted[0] == "Bearer" {
                    match AuthToken::try_from(splitted[1]) {
                        Ok(at) => return Outcome::Success(at),
                        _ => {
                            warn!("Failed to parse auth-token from header");
                        },
                    }
                }
            }
            None => {}
        }

        // test if was stored in cookies
        match request.cookies().get("token") {
            Some(token_cookie) => {
                match AuthToken::try_from(token_cookie.value()) {
                    Ok(at) => return Outcome::Success(at),
                    _ => {
                        warn!("Failed to parse auth-token from cookie");
                    },
                }
            },
            None => {}
        }

        Outcome::Forward(())
    }
}
