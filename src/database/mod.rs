use std::collections::HashMap;

use uuid::Uuid;
use postgres::{Connection, TlsMode};
use postgres::tls::native_tls::NativeTls;
use postgres::types::FromSql;
use postgres::rows::Rows;

#[derive(Debug)]
pub struct AlbumId(pub i64);

#[derive(Debug)]
pub struct Album {
    pub id: AlbumId,
    pub art_blob: Option<String>,
}

#[derive(Debug)]
pub struct AlbumMetadata {
    pub album_id: AlbumId,
    pub field_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct SongId(pub i64);

#[derive(Debug)]
pub struct Song {
    pub id: SongId,
    pub album_id: AlbumId,
    pub blob: String,
    pub length_ms: i32,
}

#[derive(Debug)]
pub struct SongMetadata {
    pub song_id: SongId,
    pub field_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct AccountSongMetadataId(pub i64);

#[derive(Debug)]
pub struct AccountSongMetadata {
    pub id: AccountSongMetadataId,
    pub account_id: Uuid,
    pub song_id: SongId,
    pub play_count: i32,
    pub score: i32,
}

#[derive(Debug)]
pub struct AccountId(Uuid);

impl AccountId {
    pub fn get_user_id(&self) -> Uuid {
        self.0.clone()
    }
}

#[derive(Debug)]
pub struct Account {
    id: AccountId,
    display_name: String,
}

#[derive(Debug)]
pub struct ForeignAccountProviderId(Uuid);

#[derive(Debug)]
pub struct ForeignAccountProvider {
    id: ForeignAccountProviderId,
    name: String,
}

#[derive(Debug)]
pub struct ForeignAccountId(Uuid);

#[derive(Debug)]
pub struct ForeignAccount {
    id: ForeignAccountId,
    provider_id: ForeignAccountProviderId,
    foreign_id: String,
    auth_token: String,

    // created_at: ...
    // last_authenticated: ...
}


use super::foreign_auth::{
    ForeignAccount as AuthForeignAccount,
};

pub fn get_conn() -> Result<Connection, Box<::std::error::Error>> {
    use std::env::var;
    let negotiator = NativeTls::new().unwrap();
    let database = var("PG_DATABASE_URL").unwrap();
    let conn = try!(Connection::connect(&database[..], TlsMode::None));
    Ok(conn)
}

fn extract_single<T: FromSql>(rows: Rows) -> Result<T, ()> {
    rows.iter().next().map(|r| r.get(0)).ok_or(())
}

fn extract_single2<T: FromSql>(rows: Rows) -> Result<T, Box<::std::error::Error>> {
    rows.iter().next().map(|r| r.get(0))
        .ok_or_else(internal_error)
}

/// returned for really unexpected errors
fn internal_error() -> Box<::std::error::Error> {
    use std::io;

    return Box::new(io::Error::new(io::ErrorKind::Other, "DB Error"));
}

pub fn find_or_create_user(connection: &Connection, acc: &AuthForeignAccount) -> Result<AccountId, Box<::std::error::Error>> {
    use std::io;
    let rows = try!(connection.query("
        SELECT fa.account_id FROM foreign_account AS fa
            WHERE
                fa.foreign_id = $1 AND
                fa.provider_id = $2
            LIMIT 1;
    ", &[&acc.account_id, &acc.provider.uuid()]));

    let mut row_iter = rows.iter();
    if let Some(row) = row_iter.next() {
        let account_id = AccountId(row.get(0));
        // try!(bump_user_login(connection, &account_id));
        return Ok(account_id);
    }

    let trans = try!(connection.transaction());

    let user_id: Uuid = {
        let rows = try!(trans.query("
            INSERT INTO account (display_name) VALUES ('') RETURNING id;
        ", &[]));
        try!(extract_single2(rows))
    };
    try!(trans.execute("
        INSERT INTO foreign_account (account_id, foreign_id, provider_id)
        VALUES ($1, $2, $3)
    ", &[&user_id, &acc.account_id, &acc.provider.uuid()]));

    try!(trans.commit());
    
    Ok(AccountId(user_id))
}
