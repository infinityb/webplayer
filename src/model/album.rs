use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct AlbumId(pub i64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Album {
    pub id: AlbumId,
    pub art_blob: Option<String>,
    pub metadata: BTreeMap<String, String>,
}