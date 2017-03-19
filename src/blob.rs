use std::str::FromStr;
use std::fmt;

use rocket::request::FromParam;
use ::util::{dehex_fixed_size, DehexError};

/// sha256
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BlobId([u8; 32]);

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for by in self.0.iter() {
            write!(f, "{:02x}", *by)?;
        }
        Ok(())
    }
}

impl FromStr for BlobId {
    type Err = DehexError;

    fn from_str(val: &str) -> Result<BlobId, DehexError> {
        let mut buf: [u8; 32] = [0; 32];
        let remaining = dehex_fixed_size(val, &mut buf[..])?;
        if remaining.len() > 0 {
            return Err(DehexError::TooLong);
        }
        Ok(BlobId(buf))
    }
}

impl<'a> FromParam<'a> for BlobId {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<BlobId, &'a str> {
        param.parse().map_err(|_| param)
    }
}
