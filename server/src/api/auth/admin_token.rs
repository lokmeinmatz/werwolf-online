use jsonwebtoken as jwt;
use jsonwebtoken::{TokenData, Validation};
use rocket::request::{Outcome};
use rocket::{request, Request};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom};
use log::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct AdminAuthClaims {
    exp: u64
}

#[derive(Debug)]
pub enum AuthClaimError {
    NoAuthCookie,
    Expired,
    Blocked,
}


pub fn gen_admin_jwt() -> Result<String, ()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let norole = AdminAuthClaims {
        // token expires after 4h
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ())?
            .as_secs()
            + 3600 * 4
    };

    jwt::encode(&jwt::Header::default(), &norole, super::DEV_SECRET.as_bytes()).map_err(|_|
        ())
}

#[derive(Debug)]
pub struct AdminToken();


impl std::convert::From<AdminAuthClaims> for AdminToken {
    fn from(aac: AdminAuthClaims) -> Self {
        AdminToken()
    }
}

impl TryFrom<&str> for AdminToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // try to decode jwt
        let tdata: TokenData<AdminAuthClaims> =
            jwt::decode(value, super::DEV_SECRET.as_bytes(), &Validation::default()).map_err(|e| {
                warn!("Deconding of jwt failed: {}", e.to_string());
                ()
            })?;
        Ok(tdata.claims.into())
    }
}


impl<'a, 'r> request::FromRequest<'a, 'r> for AdminToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(hdr) => {
                let splitted: Vec<&str> = hdr.split(" ").collect();
                info!("got Authorization header: {:?}", splitted);
                if splitted.len() == 2 && splitted[0] == "Bearer" {
                    match AdminToken::try_from(splitted[1]) {
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
        match request.cookies().get("admintoken") {
            Some(token_cookie) => {
                match AdminToken::try_from(token_cookie.value()) {
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
