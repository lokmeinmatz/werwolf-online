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
    pub user_name: String,
    pub session_id: SessionID,
    pub role: String,
    pub state: String,
}

pub enum PlayerState {
    Waiting,
    Alive,
    Dead,
    Spectator,
}

impl PlayerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            &PlayerState::Waiting => "waiting",
            &PlayerState::Alive => "alive",
            &PlayerState::Dead => "dead",
            &PlayerState::Spectator => "spectator",
        }
    }
}

impl PlayerAuthToken {
    pub fn user_name(&self) -> &str {
        self.basic.claims().user_name.as_ref().unwrap().as_str()
    }

    pub fn role(&self) -> &str {
        self.basic.claims().role.as_ref().unwrap().as_str()
    }

    pub fn get_jwt(
        session_id: SessionID,
        user_name: String,
        role: String,
        state: PlayerState,
    ) -> String {
        use std::time::UNIX_EPOCH;

        let norole = AuthClaims {
            // token expires after 4h
            exp: UNIX_EPOCH
                .elapsed()
                .unwrap() // fails if time is before UNIX_EPOCH
                .as_secs()
                + 3600 * 4,
            session_id: Some(session_id.as_str().to_owned()),
            user_name: Some(user_name),
            auth_level: "player".to_string(),
            state: Some(state.as_str().to_owned()),
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
            return Err("No role");
        }
        if value.claims().state.is_none() {
            return Err("No state");
        }

        Ok(PlayerAuthToken {
            session_id: SessionID::try_from(
                value
                    .claims()
                    .session_id
                    .as_ref()
                    .ok_or("No session_id")?
                    .as_str(),
            )?,
            user_name: value.claims().user_name.as_ref().unwrap().clone(),
            role: value.claims().role.as_ref().unwrap().clone(),
            state: value.claims().state.as_ref().unwrap().clone(),
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
