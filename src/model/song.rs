use std::collections::BTreeMap;
use super::album::{
    Album,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct SongId(pub i64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: SongId,
    pub blob: String,
    pub length_ms: i32,
    pub track_no: i16,
    pub metadata: BTreeMap<String, String>,
    pub album: Album,
}

