use std::collections::HashMap;

extern crate postgres;
extern crate ogg;
use postgres::{Connection, TlsMode};
use postgres::types::FromSql;
use postgres::rows::Rows;


#[derive(Debug)]
pub struct SongId(pub i64);

#[derive(Debug)]
pub struct AlbumId(pub i64);


/// returned for really unexpected errors
fn internal_error() -> Box<::std::error::Error> {
    use std::io;

    return Box::new(io::Error::new(io::ErrorKind::Other, "DB Error"));
}

fn extract_single2<T: FromSql>(rows: Rows) -> Result<T, Box<::std::error::Error>> {
    rows.iter().next().map(|r| r.get(0))
        .ok_or_else(internal_error)
}

pub fn create_album(
    connection: &Connection,
    ac: &AlbumCreate
) -> Result<AlbumId, Box<::std::error::Error>> {
    let trans = try!(connection.transaction());

    let album = {
        let album_rows = try!(trans.query("
            INSERT INTO album (art_blob)
            VALUES ($1)
            RETURNING id
        ", &[&ac.art_blob]));
        
        AlbumId(try!(extract_single2(album_rows)))
    };

    for (key, val) in ac.metadata.iter() {
        try!(trans.execute("
            INSERT INTO album_metadata (album_id, field_name, value)
            VALUES ($1, $2, $3)
            ON CONFLICT ON CONSTRAINT album_metadata_pkey DO UPDATE
        ", &[&album.0, key, val]));
    }

    let mut song_ids = Vec::new();
    for song in ac.songs.iter() {
        let song_rows = try!(trans.query("
            INSERT INTO song (blob, album_id, length_ms)
            VALUES ($1, $2, $3)
            RETURNING id
        ", &[&song.blob, &album.0, &song.length_ms]));

        let song = SongId(try!(extract_single2(song_rows)));
        for (key, val) in ac.metadata.iter() {
            try!(trans.execute("
                INSERT INTO song_metadata (song_id, field_name, value)
                VALUES ($1, $2, $3)
                ON CONFLICT ON CONSTRAINT song_metadata_pkey DO UPDATE
            ", &[&song.0, key, val]));
        }
        song_ids.push(song);
    }

    try!(trans.commit());

    Ok(AlbumId(0))
}

pub struct SongCreate {
    pub blob: String,
    pub length_ms: i64,
    pub metadata: HashMap<String, String>,
}

pub struct AlbumCreate {
    pub songs: Vec<SongCreate>,
    pub art_blob: Option<String>,
    pub metadata: HashMap<String, String>,   
}

pub fn get_conn() -> Result<Connection, Box<::std::error::Error>> {
    use std::env::var;
    let database = var("PG_DATABASE_URL").unwrap();
    let conn = try!(Connection::connect(&database[..], TlsMode::None));
    Ok(conn)
}

fn main() {
    let conn = get_conn().unwrap();

    let mut songs = Vec::new();
    let mut album_metadata = HashMap::new();



    let mut album = AlbumCreate {
        songs: songs,
        art_blob: None,
        metadata: album_metadata,
    };

    create_album(&conn, &album).unwrap();
}