use hyper;
use serde_json;
use uuid::Uuid;

mod google;
pub use self::google::{
    GoogleAuthProvider,
    GoogleAuthToken,
};

#[derive(Debug)]
pub struct Provider {
    // TODO: replace with const UUID
    pub id: &'static str,
    pub name: &'static str,
}

impl Provider {
    pub fn uuid(&self) -> Uuid {
        Uuid::parse_str(self.id).unwrap()
    }
}

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
pub struct ForeignAccount {
    pub provider: &'static Provider,
    pub account_id: String,
}

pub trait ForeignAuthProvider {
    type Token;

    fn authenticate(&self, token: &Self::Token) -> Result<ForeignAccount, AuthError>;
}