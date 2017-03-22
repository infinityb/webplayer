use std::io;
use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::fs::File;

use serde_json;
use url::Url;
use uuid::Uuid;
use super::{DbConnector, SongQuery};
use ::model::SongSegment;
use ::database::{
    Song,
    SongId,
    Album,
    AlbumId,
    AccountId,
};
use ::foreign_auth::{
    ForeignAccount as AuthForeignAccount,
};

pub const DRIVER_NAME: &'static str = "mock";

pub fn get_conn(url: &str) -> io::Result<MockConnector> {
    let url = Url::parse(url)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        ?;

    let data_dir = PathBuf::from(format!(".{}", url.path()));
    let database_path = data_dir.join("database.json");

    let mut db_json = File::open(&database_path)?;
    let fixtures: Fixtures = serde_json::from_reader(&mut db_json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        ?;
    
    return Ok(MockConnector {
        base_path: data_dir,
        songs: fixtures.songs.clone(),
        albums: fixtures.albums.clone(),
        accounts: HashMap::new(),
    });
}

#[derive(Debug, Deserialize)]
struct Fixtures {
    albums: Vec<RawAlbum>,
    songs: Vec<RawSong>,
}

pub struct MockConnector {
    base_path: PathBuf,
    songs: Vec<RawSong>,
    albums: Vec<RawAlbum>,
    accounts: HashMap<String, AccountId>,
}

impl DbConnector for MockConnector {
    fn get_songs(&self, query: &SongQuery) -> io::Result<Vec<Song>>
    {
        let mut out = Vec::new();
        for song in self.songs.iter() {
            out.push(song.cook(self)?);
        }
        Ok(out)
    }

    fn find_or_create_user(&mut self, acc: &AuthForeignAccount) -> io::Result<AccountId>
    {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let name = format!("{}@{}", acc.provider.name, acc.account_id);
        let (created, accid) = match self.accounts.entry(name) {
            Occupied(mut occ) => (false, occ.into_mut()),
            Vacant(mut vac) => (true, vac.insert(AccountId(Uuid::new_v4()))),
        };
        if created {
            // changed - lets write out the new data
        }
        Ok(accid.clone())
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawAlbum {
    pub id: AlbumId,
    pub art_blob: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawSong {
    pub id: SongId,
    pub blob: String,
    pub length_ms: i32,
    // pub track_no: i16,
    pub metadata: BTreeMap<String, String>,
    pub album_id: AlbumId,
    pub segments: Option<Vec<SongSegment>>,
}

impl RawAlbum
{
    fn cook(&self, conn: &MockConnector)
        -> io::Result<Album>
    {
        Ok(Album {
            id: self.id.clone(),
            art_blob: self.art_blob.clone(),
            metadata: self.metadata.clone(),
        })
    }
}

impl RawSong
{
    fn cook(&self, conn: &MockConnector)
        -> io::Result<Song>
    {
        let album = conn.albums
            .iter()
            .filter(|a| a.id == self.album_id)
            .nth(0)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, format!("unknown album {}", self.album_id.0))
            })
            ?.cook(conn)?;
        
        Ok(Song {
            id: self.id.clone(),
            blob: self.blob.clone(),
            length_ms: self.length_ms,
            // track_no: self.track_no,
            metadata: self.metadata.clone(),
            segments: self.segments.clone(),
            album: album,
        })
    }
}