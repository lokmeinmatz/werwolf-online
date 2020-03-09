use crate::api::auth::token::{AuthClaims, AuthLevel, BasicAuthToken};
use crate::api::auth::SessionID;
use rocket::{request, Request};
use std::convert::TryFrom;

use jsonwebtoken as jwt;
use log::{info, warn};
use rocket::http::RawStr;
use rocket::request::Outcome;

#[derive(Debug)]
pub struct PlayerAuthToken {
    pub basic: BasicAuthToken,
    pub user_id: u32,
    pub session_id: SessionID
}

impl PlayerAuthToken {
    pub fn user_name(&self) -> &str {
        self.basic.claims().user_name.as_ref().unwrap().as_str()
    }

    pub fn role(&self) -> &str {
        self.basic.claims().role.as_ref().unwrap().as_str()
    }

    pub fn get_jwt(user_id: u32, session_id: SessionID, user_name: String, role: String) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let norole = AuthClaims {
            // token expires after 4h
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap() // fails if time is before UNIX_EPOCH
                .as_secs()
                + 3600 * 4,
            user_id: Some(user_id),
            session_id: Some(session_id.as_str().to_owned()),
            user_name: Some(user_name),
            auth_level: "player".to_string(),
            role: Some(role),
        };

        jwt::encode(
            &jwt::Header::default(),
            &norole,
            super::DEV_SECRET.as_bytes(),
        )
        .unwrap()
    }
}

impl<'a> TryFrom<BasicAuthToken> for PlayerAuthToken {
    type Error = &'static str;

    fn try_from(value: BasicAuthToken) -> Result<Self, Self::Error> {
        if value.auth_level() != AuthLevel::Player {
            return Err("No player auth_level");
        }

        if value.claims().user_name.is_none() {
            return Err("No user_name");
        }

        if value.claims().role.is_none() {
            return Err("No role")
        }

        Ok(PlayerAuthToken {
            user_id: value.claims().user_id.ok_or("No user_id")?,
            session_id: SessionID::try_from(
                value.claims().session_id.as_ref().ok_or("No session_id")?.as_str(),
            )?,
            basic: value,
        })
    }
}

impl TryFrom<&str> for PlayerAuthToken {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let basic = BasicAuthToken::try_from(value)?;

        match basic.auth_level() {
            AuthLevel::Player => PlayerAuthToken::try_from(basic),
            _ => Err("Wrong auth_level"),
        }
    }
}

impl<'r> request::FromFormValue<'r> for PlayerAuthToken {
    type Error = &'static str;

    fn from_form_value(form_value: &'r RawStr) -> Result<Self, Self::Error> {
        // first into BasicAuthToken
        Self::try_from(form_value.as_str())
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for PlayerAuthToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(hdr) => {
                let splitted: Vec<&str> = hdr.split(" ").collect();
                info!("got Authorization header: {:?}", splitted);
                if splitted.len() == 2 && splitted[0] == "Bearer" {
                    match PlayerAuthToken::try_from(splitted[1]) {
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
            Some(token_cookie) => match PlayerAuthToken::try_from(token_cookie.value()) {
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
