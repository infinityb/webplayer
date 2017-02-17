use std::collections::BTreeMap;
use super::super::model::Song;
use super::StagedBlob;

pub struct SongCreate {
    pub blob: StagedBlob,
    pub metadata: BTreeMap<String, String>,
}

pub struct AlbumCreateRequest {
    pub songs: Vec<SongCreate>,
}

pub struct AlbumCreateResponse {
    pub songs: Vec<Song>,
}
