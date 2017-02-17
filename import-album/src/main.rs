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
    println!("cp0");

    for (key, val) in ac.metadata.iter() {
        try!(trans.execute("
            INSERT INTO album_metadata (album_id, field_name, value)
            VALUES ($1, $2, $3)
            -- ON CONFLICT ON CONSTRAINT album_metadata_pkey DO UPDATE
        ", &[&album.0, key, val]));
        println!("cp1 - {}", key);
    }
    println!("cp1");

    let mut song_ids = Vec::new();
    for song in ac.songs.iter() {
        let song_rows = try!(trans.query("
            INSERT INTO song (blob, track_no, album_id, length_ms)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        ", &[&song.blob, &song.track_no, &album.0, &song.length_ms]));
        println!("cp2");

        let dbsong = SongId(try!(extract_single2(song_rows)));
        for (key, val) in song.metadata.iter() {
            try!(trans.execute("
                INSERT INTO song_metadata (song_id, field_name, value)
                VALUES ($1, $2, $3)
                -- ON CONFLICT ON CONSTRAINT song_metadata_pkey DO UPDATE
            ", &[&dbsong.0, key, val]));
            println!("cp2.0 - {}", key);
        }

        song_ids.push(dbsong);
        println!("cp2 - {}", song_ids.len());
    }
    println!("cp3");

    try!(trans.commit());

    Ok(AlbumId(0))
}

pub struct SongCreate {
    pub blob: String,
    pub track_no: i16,
    pub length_ms: i32,
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

    let mut album_metadata = HashMap::new();
    album_metadata.insert("album".into(), "Senki Zesshou Symphogear Character Song Series 4 - Yukine Chris".into());
    album_metadata.insert("artist".into(), "Takagaki Ayahi".into());
    album_metadata.insert("COMMENT".into(), "Exact Audio Copy".into());
    album_metadata.insert("DATE".into(), "2012-03-28".into());
    album_metadata.insert("GENRE".into(), "Anime".into());
    album_metadata.insert("TRACKTOTAL".into(), "4".into());
    
    let mut track01 = SongCreate {
        blob: "5b6ff8bc3ef0110d61ccdefc7178bccb8184c6321ccb83537806ddae7d21821e".into(),
        track_no: 1,
        length_ms: 222773,
        metadata: HashMap::new(),
    };
    track01.metadata.insert("title".into(), "Makyuu Isshi-Bal".into());
    track01.metadata.insert("tracknumber".into(), "1".into());
    let track01 = track01;
    
    let mut track02 = SongCreate {
        blob: "3bba03c8437be25542968bd3ce838f34985ca5249b11901e8cc57e784811456b".into(),
        track_no: 2,
        length_ms: 239986,
        metadata: HashMap::new(),
    };
    track02.metadata.insert("title".into(), "Tsunaida Te Dake ga Tsumugu mono".into());
    track02.metadata.insert("tracknumber".into(), "2".into());
    let track02 = track02;

    let mut track03 = SongCreate {
        blob: "9605f1a07cacb7fda954a4c698e7bf1c6798813cf1be1d8687f4946ddefcc9e6".into(),
        track_no: 3,
        length_ms: 222106,
        metadata: HashMap::new(),
    };
    track03.metadata.insert("title".into(), "Makyuu Isshi-Bal (off vocal)".into());
    track03.metadata.insert("tracknumber".into(), "3".into());
    let track03 = track03;

    let mut track04 = SongCreate {
        blob: "8a1b641a12b0b007a51d5a9dd9728f1f7b73551d500ec7060d06d34ac44d79b5".into(),
        track_no: 4,
        length_ms: 239319,
        metadata: HashMap::new(),
    };
    track04.metadata.insert("title".into(), "Tsunaida Te Dake ga Tsumugu mono (off vocal)".into());
    track04.metadata.insert("tracknumber".into(), "4".into());
    let track04 = track04;

    let album = AlbumCreate {
        songs: vec![track01, track02, track03, track04],
        art_blob: None,
        metadata: album_metadata,
    };

    create_album(&conn, &album).unwrap();
}