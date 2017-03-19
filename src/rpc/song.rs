use super::super::model::Song;

#[derive(Serialize)]
pub struct SongSetResponse {
    pub results: Vec<Song>,
}