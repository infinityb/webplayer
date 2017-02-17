use std::collections::{BTreeMap, HashMap};

use uuid::Uuid;
use postgres::{Connection, TlsMode};
use postgres::tls::native_tls::NativeTls;
use postgres::types::FromSql;
use postgres::rows::Rows;

use ::util::json::JsonDocument;
use ::model::{AlbumId, Album, SongId, Song};

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
}

use super::foreign_auth::{
    ForeignAccount as AuthForeignAccount,
};

pub fn get_conn(dburl: &str) -> Result<Connection, Box<::std::error::Error>> {
    let negotiator = NativeTls::new().unwrap();
    let conn = try!(Connection::connect(dburl, TlsMode::None));
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

pub fn get_songs(connection: &Connection) -> Result<impl Iterator<Item=Song>, Box<::std::error::Error>>
{
    let rows = try!(connection.query("
        SELECT
            s.id AS song_id,
            s.blob AS song_blob,
            s.length_ms AS song_length_ms,
            s.track_no AS song_track_no,
            (
                SELECT jsonb_object_agg(sm.field_name, sm.value) AS song_metadata
                FROM song_metadata AS sm WHERE sm.song_id = s.id
            ) AS song_metadata,
            s.album_id AS album_id,
            (SELECT a.art_blob FROM album AS a WHERE s.album_id = a.id) AS album_art_blob,
            (
                SELECT jsonb_object_agg(am.field_name, am.value) AS album_metadata
                FROM album_metadata AS am WHERE am.album_id = s.album_id
            ) AS AS album_metadata
        FROM song AS s
    ", &[]));
    let mut out = Vec::new();
    for row in rows.iter() {
        let album = Album {
            id: AlbumId(row.get(5)),
            art_blob: row.get(6),
            metadata: row.get::<_, JsonDocument>(7).deserialize()?,
        };
        out.push(Song {
            id: SongId(row.get(0)),
            album: album,
            blob: row.get(1),
            length_ms: row.get(2),
            track_no: row.get(3),
            metadata: row.get::<_, JsonDocument>(4).deserialize()?,
        });
    }
    Ok(out.into_iter())
}

