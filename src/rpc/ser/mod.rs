use serde;
use serde_json;

mod song_object;

pub use self::song_object::SongObject;

pub fn to_json_value<S: ?Sized>(s: &S)
    -> serde_json::Result<serde_json::Value>
    where S: serde::ser::Serialize
{
    // this is horrible, sorry.
    // perhaps we could serialize into a flatbuffer that serializes into JSON
    let bytes = serde_json::to_vec(s)?;
    serde_json::from_slice(&bytes)
}