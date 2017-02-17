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

struct Error {
    //
}

// rocket enforces the existence atm so this one is harder.
//
// {"error": {
//    "type": "missing-access-token",
//    "message" "Missing acess token"
// }}

// {"error": {
//    "type": "invalid-access-token",
//    "message" "Invalid or expired acess token"
// }}
