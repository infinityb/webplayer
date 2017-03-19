use std::io;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use uuid::Uuid;
use postgres::{Connection, TlsMode};
use postgres::tls::native_tls::NativeTls;
use postgres::types::FromSql;
use postgres::rows::Rows;

use ::util::json::JsonDocument;
use ::model::{AlbumId, Album, SongId, Song};
use super::{DbConnector, SongQuery};

use ::foreign_auth::{
    ForeignAccount as AuthForeignAccount,
};
use super::super::{
    AccountId,
};

pub const DRIVER_NAME: &'static str = "postgresql";

pub struct PostgresConnector {
    pgconn: Connection,
}

impl PostgresConnector {
    pub fn connect(dburl: &str) -> io::Result<PostgresConnector> {
        let pgconn = Connection::connect(dburl, TlsMode::None)
            .map_err(adapt_error_tagged("Failed to connect to database"))
            ?;
        
        Ok(PostgresConnector {
            pgconn: pgconn,
        })
    }
}

impl DbConnector for PostgresConnector {
    fn get_songs(&self, query: &SongQuery) -> io::Result<Vec<Song>>
    {
        let rows = try!(self.pgconn.query("
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
                ) AS album_metadata
            FROM song AS s
        ", &[]));
        let mut out = Vec::new();
        for row in rows.iter() {
            let album = Album {
                id: AlbumId(row.get(5)),
                art_blob: row.get(6),
                metadata: {
                    row.get::<_, Option<JsonDocument>>(7)
                        .unwrap_or_else(JsonDocument::empty)
                        .deserialize()
                        .map_err(adapt_error_tagged("error deserializing json"))
                        ?
                }
            };
            out.push(Song {
                id: SongId(row.get(0)),
                album: album,
                blob: row.get(1),
                length_ms: row.get(2),
                track_no: row.get(3),
                metadata: {
                    row.get::<_, Option<JsonDocument>>(4)
                        .unwrap_or_else(JsonDocument::empty)
                        .deserialize()
                        .map_err(adapt_error_tagged("error deserializing json"))
                        ?
                }
            });
        }
        Ok(out)
    }

    fn find_or_create_user(&mut self, acc: &AuthForeignAccount) -> io::Result<AccountId> {
        let rows = try!(self.pgconn.query("
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

        let trans = try!(self.pgconn.transaction());

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
}

use std::boxed::FnBox;

fn adapt_error_tagged<'a, E: Error>(tag: &'a str) -> Box<FnBox(E) -> io::Error + 'a> {
    Box::new(move |e| {
        io::Error::new(io::ErrorKind::Other, format!("{}: {}", tag, e))
    })
}

fn extract_single<T: FromSql>(rows: Rows) -> Result<T, ()> {
    rows.iter().next().map(|r| r.get(0)).ok_or(())
}

fn extract_single2<T: FromSql>(rows: Rows) -> io::Result<T> {
    rows.iter().next().map(|r| r.get(0))
        .ok_or_else(internal_error)
}

/// returned for really unexpected errors
fn internal_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "DB Error")
}
