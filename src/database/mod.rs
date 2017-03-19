use std::collections::{BTreeMap, HashMap};
use std::io;

use uuid::Uuid;
use postgres::{Connection, TlsMode};
use postgres::tls::native_tls::NativeTls;
use postgres::types::FromSql;
use postgres::rows::Rows;

use ::util::json::JsonDocument;
use ::model::{AlbumId, Album, SongId, Song};

pub mod drivers;

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

#[derive(Debug, Clone)]
pub struct AccountId(Uuid);

impl AccountId {
    pub fn get_user_id(&self) -> Uuid {
        self.0.clone()
    }
}

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
}

pub struct SongQuery {
    //
}