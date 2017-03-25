use std::io::{self, Read, Seek};
use std::path::PathBuf;
use std::fs::File;

use ::blob::BlobId;

#[derive(Deserialize)]
pub struct AppConfig {
    pub secret: String,
    pub google_auth: GoogleAuthConfig,
    pub database: DatabaseConfig,
    pub vfs_driver: VfsDriverConfig,
    pub web: WebConfig,
}

#[derive(Deserialize)]
pub struct GoogleAuthConfig {
    pub audience: String,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub read_url: Option<String>,
    pub write_url: String,
}

impl DatabaseConfig {
    pub fn read_url(&self) -> &str
    {
        if let Some(ref url) = self.read_url {
            return url;
        }
        &self.write_url
    }

    pub fn write_url(&self) -> &str
    {
        &self.write_url
    }
}

#[derive(Deserialize)]
#[serde(tag="driver_name")]
pub enum VfsDriverConfig
{
    #[serde(rename="blob")]
    Blob(BlobDriver),
}

#[derive(Deserialize, Clone)]
pub struct BlobDriver
{
    pub blob_base: PathBuf,
}

impl BlobDriver
{
    fn create(&self) -> BlobDriver
    {
        self.clone()
    }
}

impl VfsDriverConfig
{
    pub fn boxed(&self) -> Box<VfsBackend>
    {
        match *self {
            VfsDriverConfig::Blob(ref cfg) => Box::new(cfg.create()),
        }
    }
}

pub trait VfsBackend
{
    fn open_read(&self, blob_id: &BlobId) -> io::Result<File>;
}

impl VfsBackend for BlobDriver
{
    fn open_read(&self, blob_id: &BlobId) -> io::Result<File>
    {
        let hash = format!("{}", blob_id);
        let path = self.blob_base.join(&hash[0..2]).join(&hash);
        println!("attempting to open path {}", path.display());
        let file = try!(File::open(&path));
        Ok(file)
    }
}

#[derive(Deserialize)]
pub struct WebConfig {
    pub allow_origins: Vec<String>,
}