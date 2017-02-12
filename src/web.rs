use std::str::FromStr;
use rocket::request::FromRequest;
use rocket::{Request, Outcome};
use rocket::http::Status;

#[derive(Debug)]
pub struct AuthToken {
    value: String,
}

impl AuthToken {
    pub fn is_valid(&self) -> bool {
        self.value.len() == 21
    }
}

impl FromStr for AuthToken {
    type Err = ();
    
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(AuthToken { value: val.to_owned() })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        for token in request.headers().get("Authorization") {
            if let Some(bearer) = deprefix("bearer ", token) {
                if let Ok(tok) = bearer.parse() {
                    return Outcome::Success(tok);
                }
            }
        }
        Outcome::Failure((Status::Forbidden, ()))
    }
}

fn deprefix<'a>(prefix: &'static str, value: &'a str) -> Option<&'a str> {
    if value.starts_with(prefix) {
        Some(&value[prefix.len()..])
    } else {
        None
    }
}