mod login;
pub use self::login::{LoginRequest, LoginResponse};

mod blob;
pub use self::blob::{
    BlobUploadResponse,
    StagedBlob,
};

mod album;
pub use self::album::{
    AlbumCreateRequest,
    AlbumCreateResponse,
};

pub mod cursor;
pub mod hateoas;
pub use self::hateoas::HateoasCollection;

mod hateoas_def; // traits only
pub mod query;
pub mod ser;

struct Error {
    //
}