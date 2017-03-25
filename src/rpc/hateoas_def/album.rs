use serde_json;

use super::super::hateoas::HateoasObject;
use super::super::ser::{to_json_value}; // , AlbumObject};
use model::Album;

impl HateoasObject for Album {
    fn relative_url(&self) -> String {
        format!("/album/{}", self.id.0)
    }

    fn serialize(&self) -> serde_json::Result<serde_json::Value> {
        // just do it raw for now
        to_json_value(self)
    }

    fn dependencies(&self) -> Vec<Box<HateoasObject>> {
        Vec::new()
    }

    fn rels(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}