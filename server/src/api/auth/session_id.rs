use rocket::{http, request};
use std::convert::TryFrom;
use std::fmt::Error;
use std::fmt::Formatter;
use log::{error, warn, info};

const SID_LENGTH: usize = 8;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SessionID([u8; SID_LENGTH]);

impl SessionID {
    // when the SessionID is created, the content is allready validated
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl TryFrom<&str> for SessionID {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != SID_LENGTH {
            return Err("Invalid id-length");
        }
        if !value.chars().all(|c| {
            (c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_uppercase()))
                && c.len_utf8() == 1
        }) {
            return Err("session-id must consist of digits and uppercase ascii letters");
        }
        let mut buf = ['-' as u8; SID_LENGTH];
        for (i, c) in value.chars().enumerate() {
            buf[i] = c as u8;
        }
        Ok(SessionID(buf))
    }
}

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
        info!("sid: {}", sid);
        if !sid
            .chars()
            .all(|c| c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_uppercase()))
        {
            return Err("sessionID must by uppercase letters or digits (radix 10)");
        }

        let mut sid_chars = ['-' as u8; SID_LENGTH];
        for (i, c) in sid.chars().enumerate() {
            sid_chars[i] = c as u8;
        }
        Ok(SessionID(sid_chars))
    }
}
