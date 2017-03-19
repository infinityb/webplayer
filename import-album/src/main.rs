use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read};

extern crate postgres;
extern crate ogg;
extern crate crypto;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use postgres::{Connection, TlsMode};
use postgres::types::FromSql;
use postgres::rows::Rows;
use crypto::digest::Digest;
use crypto::sha2::Sha256;


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

#[derive(Serialize, Debug)]
pub struct SongCreate {
    pub blob: String,
    pub track_no: i16,
    pub length_ms: i32,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Debug)]
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

fn remove_album_meta(meta: &HashMap<String, String>, songs: &mut [SongCreate])
{
    for song in songs.iter_mut() {
        for key in meta.keys() {
            song.metadata.remove(key);
        }
    }
}

fn album_from_songs(mut songs: Vec<SongCreate>) -> AlbumCreate {
    let metadata = unified_metadata(&songs);
    remove_album_meta(&metadata, &mut songs);

    AlbumCreate {
        songs: songs,
        art_blob: None,
        metadata: metadata,
    }
}

fn unified_metadata(ss: &[SongCreate]) -> HashMap<String, String>
{
    let mut song_iter = ss.iter();
    let mut min = match song_iter.next() {
        Some(song) => song.metadata.clone(),
        None => return HashMap::new(),
    };
    for song in song_iter {
        let mut remove_keys = Vec::new();
        for (key, val) in min.iter() {
            match song.metadata.get(key) {
                Some(val_cand) => {
                    if val_cand != val {
                        remove_keys.push(key.clone());
                    }
                },
                None => {
                    remove_keys.push(key.clone());
                }
            }
        }
        for key in remove_keys.into_iter() {
            min.remove(&key);
        }
    }
    min
}

fn main() {
    use std::env::args_os;
    use std::fs::read_dir;

    let conn = get_conn().unwrap();

    let dir = args_os().nth(1).unwrap();

    let mut files = Vec::new();
    let mut songs = Vec::new();
    for entry in read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            files.push(entry.path());
        }
    }

    let mut blobs = HashMap::new();
    let mut track_num = 1;
    for file in files.iter() {
        let mut buf = Vec::new();
        let mut ff = File::open(&file).unwrap();
        ff.read_to_end(&mut buf).unwrap();

        let mut hasher = Sha256::new();
        hasher.input(&buf);
        let blob_hash = hasher.result_str();

        let track = match ogg::OggTrack::new(&buf) {
            Ok(track) => track,
            Err(ogg::OggPageCheckError::BadCapture) => continue,
            Err(err) => panic!("{:?}", err),
        };
        
        blobs.insert(blob_hash.clone(), buf.clone());

        let idx = track_num;
        track_num += 1;

        let mut page_iter = track.pages();
        let ident = ogg::vorbis::VorbisPacket::find_identification(&mut page_iter).unwrap();
        let id_header = ident.identification_header().unwrap();
        let sample_rate = id_header.audio_sample_rate;
        let comments = ogg::vorbis::VorbisPacket::find_comments(&mut page_iter).unwrap();
        let comments = comments.comments().unwrap().comments;

        let mut granule_pos_max = 0;
        for page in track.pages() {
            if granule_pos_max < page.position() {
                granule_pos_max = page.position();
            }
        }

        songs.push(SongCreate {
            blob: blob_hash,
            track_no: idx as i16,
            length_ms: (1000 * granule_pos_max / sample_rate as u64) as i32,
            metadata: comments.into_iter().collect(),
        });        
    }

    let album = album_from_songs(songs);
    println!("{}", serde_json::to_string_pretty(&album).unwrap());

    for (key, blob) in blobs.iter() {
        use std::fs::OpenOptions;

        let filename = format!("xx/blob/{}/{}", &key[0..2], key);
        let mut out = OpenOptions::new().create(true).write(true).open(&filename).unwrap();
        out.write_all(blob).unwrap();
    }

    create_album(&conn, &album).unwrap();
}