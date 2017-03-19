use std::io;

pub mod postgres;
pub mod mock;

use ::database::{
    Song,
    SongQuery,
    AccountId,
};
use ::foreign_auth::{
    ForeignAccount as AuthForeignAccount,
};

pub trait DbConnector {
    fn get_songs(&self, query: &SongQuery) -> io::Result<Vec<Song>>;

    fn find_or_create_user(&mut self, acc: &AuthForeignAccount) -> io::Result<AccountId>;
}

pub fn get_driver(url_raw: &str) -> io::Result<Box<DbConnector>> {
    use url::{Url, ParseError};

    let url = Url::parse(url_raw)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error parsing url: {}", e)))?;

    println!("getting driver {}", url.scheme());
    match url.scheme() {
        postgres::DRIVER_NAME => Ok(Box::new(postgres::PostgresConnector::connect(url_raw)?)),
        mock::DRIVER_NAME => Ok(Box::new(mock::get_conn(url_raw)?)),
        _ => Err(unknown_scheme(url.scheme())),
    }
}

fn unknown_scheme(scheme: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("unknown scheme: {}", scheme))
}