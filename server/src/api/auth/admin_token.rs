use crate::api::auth::token::{AuthClaims, AuthLevel, BasicAuthToken};
use jsonwebtoken as jwt;
use log::{info, warn};
use rocket::http::RawStr;
use rocket::request::Outcome;
use rocket::{request, Request};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct AdminAuthToken {
    basic: BasicAuthToken,
}

impl AdminAuthToken {
    pub fn get_jwt() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let norole = AuthClaims {
            // token expires after 4h
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap() // fails if time is before UNIX_EPOCH
                .as_secs()
                + 3600 * 4,
            session_id: None,
            user_name: None,
            role: None,
            state: None,
            auth_level: "control".to_string(),
            user_id: None
        };

        jwt::encode(
            &jwt::Header::default(),
            &norole,
            super::DEV_SECRET.as_bytes(),
        )
        .unwrap()
    }
}

impl TryFrom<BasicAuthToken> for AdminAuthToken {
    type Error = &'static str;

    fn try_from(value: BasicAuthToken) -> Result<Self, Self::Error> {
        if value.auth_level() != AuthLevel::Control {
            return Err("No control auth_level");
        }

        Ok(AdminAuthToken { basic: value })
    }
}

impl TryFrom<&str> for AdminAuthToken {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let basic = BasicAuthToken::try_from(value)?;
        info!("basic: {:?}", basic);
        AdminAuthToken::try_from(basic)
    }
}

impl<'r> request::FromFormValue<'r> for AdminAuthToken {
    type Error = &'static str;

    fn from_form_value(form_value: &'r RawStr) -> Result<Self, Self::Error> {
        // first into BasicAuthToken
        Self::try_from(form_value.as_str())
    }
}

impl<'a, 'r> request::FromRequest<'a, 'r> for AdminAuthToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(hdr) => {
                let splitted: Vec<&str> = hdr.split(" ").collect();
                info!("got Authorization header: {:?}", splitted);
                if splitted.len() == 2 && splitted[0] == "Bearer" {
                    match AdminAuthToken::try_from(splitted[1]) {
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
            Some(token_cookie) => match AdminAuthToken::try_from(token_cookie.value()) {
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
