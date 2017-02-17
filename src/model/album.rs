use std::collections::BTreeMap;

#[derive(Serialize, Debug)]
pub struct AlbumId(pub i64);

#[derive(Serialize, Debug)]
pub struct Album {
    pub id: AlbumId,
    pub art_blob: Option<String>,
    pub metadata: BTreeMap<String, String>,
}