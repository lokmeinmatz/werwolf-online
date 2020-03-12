use jsonwebtoken as jwt;
use jsonwebtoken::{TokenData, Validation};
use log::{info, warn};
use rocket::http::RawStr;
use rocket::request::Outcome;
use rocket::{request, Request};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthClaims {
    pub exp: u64,
    pub auth_level: String,
    // -- player
    pub session_id: Option<String>,
    pub user_name: Option<String>,
    pub role: Option<String>,
    pub state: Option<String>, // -- controller
}

#[derive(Debug)]
pub enum AuthClaimError {
    NoAuthCookie,
    Expired,
    Blocked,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AuthLevel {
    Player,
    Control,
}

#[derive(Debug)]
pub struct BasicAuthToken {
    exp: u64,
    auth_level: AuthLevel,
    claims: AuthClaims,
}

impl BasicAuthToken {
    pub fn auth_level(&self) -> AuthLevel {
        self.auth_level
    }

    pub fn claims(&self) -> &AuthClaims {
        &self.claims
    }
}

impl std::convert::TryFrom<AuthClaims> for BasicAuthToken {
    type Error = &'static str;

    fn try_from(value: AuthClaims) -> Result<Self, Self::Error> {
        //let sid = SessionID::try_from(value.session_id.as_str()).map_err(|_| ())?;

        let auth_level = match value.auth_level.as_str() {
            "player" => AuthLevel::Player,
            "control" => AuthLevel::Control,
            _ => {
                return Err("Unknown auth_level");
            }
        };

        Ok(BasicAuthToken {
            exp: value.exp,
            auth_level,
            claims: value,
        })

        /*Ok(AuthToken {
            exp: value.exp,
            session_id: sid,
            user_id: value.user_id,
            username: value.sub,
        })*/
    }
}

impl TryFrom<&str> for BasicAuthToken {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // try to decode jwt
        let tdata: TokenData<AuthClaims> =
            jwt::decode(value, super::DEV_SECRET.as_bytes(), &Validation::default())
                .map_err(|e| "JWT decoding failed")?;
        tdata.claims.try_into()
    }
}

impl<'r> request::FromFormValue<'r> for BasicAuthToken {
    type Error = &'static str;

    fn from_form_value(form_value: &'r RawStr) -> Result<Self, Self::Error> {
        // try to decode jwt
        let tdata: TokenData<AuthClaims> = jwt::decode(
            form_value.as_str(),
            super::DEV_SECRET.as_bytes(),
            &Validation::default(),
        )
        .map_err(|_| "No valid JWT")?;

        tdata.claims.try_into().map_err(|_| "invalid claims")
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for BasicAuthToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(hdr) => {
                let splitted: Vec<&str> = hdr.split(" ").collect();
                info!("got Authorization header: {:?}", splitted);
                if splitted.len() == 2 && splitted[0] == "Bearer" {
                    match BasicAuthToken::try_from(splitted[1]) {
                        Ok(at) => return Outcome::Success(at),
                        _ => {
                            warn!("Failed to parse auth-token from header");
                        }
                    }
                }
            }
            None => {}
        }

        // test if was stored in cookies
        match request.cookies().get("token") {
            Some(token_cookie) => match BasicAuthToken::try_from(token_cookie.value()) {
                Ok(at) => return Outcome::Success(at),
                _ => {
                    warn!("Failed to parse auth-token from cookie");
                }
            },
            None => {}
        }

        Outcome::Forward(())
    }
}
