use hyper;
use serde_json;

mod google;
pub use self::google::{
    GoogleAuthProvider,
    GoogleAuthToken,
};

#[derive(Debug)]
struct ProviderId(&'static str);

#[derive(Debug)]
pub enum AuthErrorKind {
    RemoteServiceError,
    InvalidToken,
}

#[derive(Debug)]
pub struct AuthError {
    pub kind: AuthErrorKind,
    pub message: String,
}

impl From<hyper::Error> for AuthError {
    fn from(e: hyper::Error) -> AuthError {
        AuthError {
            kind: AuthErrorKind::RemoteServiceError,
            message: format!("{}", e),
        }
    }
}

#[derive(Debug)]
struct ForeignAccount {
    provider: ProviderId,
    account_id: String,
}

pub trait ForeignAuthProvider {
    type Token;

    fn authenticate(&self, token: &Self::Token) -> Result<ForeignAccount, AuthError>;
}