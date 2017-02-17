use std::collections::BTreeMap;
use super::album::{
    Album,
};

#[derive(Serialize, Debug)]
pub struct SongId(pub i64);

#[derive(Serialize, Debug)]
pub struct Song {
    pub id: SongId,
    pub blob: String,
    pub length_ms: i32,
    pub track_no: i16,
    pub metadata: BTreeMap<String, String>,
    pub album: Album,
}