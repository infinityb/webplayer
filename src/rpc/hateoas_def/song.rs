use serde_json;

use super::super::hateoas::HateoasObject;
use super::super::ser::{to_json_value, SongObject};
use model::Song;

impl HateoasObject for Song {
    fn relative_url(&self) -> String {
        format!("/song/{}", self.id.0)
    }

    fn serialize(&self) -> serde_json::Result<serde_json::Value> {
        to_json_value(&SongObject { wrapped: self })
    }

    fn dependencies(&self) -> Vec<Box<HateoasObject>> {
        let mut out = Vec::<Box<HateoasObject>>::new();
        out.push(Box::new(self.album.clone()));
        out
    }

    fn rels(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
