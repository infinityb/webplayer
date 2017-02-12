use uuid::Uuid;

#[derive(Debug)]
pub struct AlbumId(pub i64);

#[derive(Debug)]
pub struct Album {
    pub id: AlbumId,
    pub art_blob: String,
}

#[derive(Debug)]
pub struct AlbumMetadata {
    pub album_id: AlbumId,
    pub field_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct SongId(pub i64);

#[derive(Debug)]
pub struct Song {
    pub id: SongId,
    pub album_id: AlbumId,
    pub blob: String,
    pub length_ms: i32,
}

#[derive(Debug)]
pub struct SongMetadata {
    pub song_id: SongId,
    pub field_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct AccountSongMetadataId(pub i64);

#[derive(Debug)]
pub struct AccountSongMetadata {
    pub id: AccountSongMetadataId,
    pub account_id: Uuid,
    pub song_id: SongId,
    pub play_count: i32,
    pub score: i32,
}

#[derive(Debug)]
pub struct AccountId(Uuid);

#[derive(Debug)]
pub struct Account {
    id: AccountId,
    display_name: String,
}

#[derive(Debug)]
pub struct ForeignAccountProviderId(Uuid);

#[derive(Debug)]
pub struct ForeignAccountProvider {
    id: ForeignAccountProviderId,
    name: String,
}

#[derive(Debug)]
pub struct ForeignAccountId(Uuid);

#[derive(Debug)]
pub struct ForeignAccount {
    id: ForeignAccountId,
    provider_id: ForeignAccountProviderId,
    foreign_id: String,
    auth_token: String,

    // created_at: ...
    // last_authenticated: ...
}
