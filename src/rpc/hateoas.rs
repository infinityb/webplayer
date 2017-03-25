
use std::collections::{VecDeque, BTreeMap};

use serde;
use serde::ser::Error as SError;
use serde_json;

pub trait HateoasObject
{
    fn relative_url(&self) -> String;

    fn serialize(&self) -> serde_json::Result<serde_json::Value>;

    fn dependencies(&self) -> Vec<Box<HateoasObject>>;

    fn rels(&self) -> Vec<(String, String)>;
}

pub struct HateoasCollection<T>
{
    pub endpoint: &'static str,
    pub next_token: String,
    pub curr_token: String,
    pub prev_token: String,
    pub limit: u32,
    pub items: Vec<T>,
}

impl<T> serde::ser::Serialize for HateoasCollection<T>
    where T: HateoasObject + Clone
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeMap;
    
        // the links to embedded values
        let mut items = Vec::new();
        for item in self.items.iter() {
            items.push(item.relative_url());
        }

        let mut embedded: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let mut queue: VecDeque<Box<HateoasObject>> = VecDeque::new();
        for item in self.items.iter().cloned() {
            queue.push_back(Box::new(item));
        }
        loop {
            let item = match queue.pop_front() {
                Some(v) => v,
                None => break,
            };
            let rel_url = item.relative_url();
            if embedded.get(&rel_url).is_some() {
                continue;
            }
            let value = item.serialize()
                .map_err(|e| SError::custom(format!("{}", e)))
                ?;
            embedded.insert(rel_url, value);
            for dep in item.dependencies().into_iter() {
                queue.push_back(dep);
            }
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_key("_links")?;
        map.serialize_value(&HateoasCollectionLinks { p: self });
        map.serialize_key("_items")?;
        map.serialize_value(&items)?;
        map.serialize_key("_embedded")?;
        map.serialize_value(&embedded)?;
        map.end()
    }
}

struct HateoasCollectionLinks<'a, T: 'a>
{
    p: &'a HateoasCollection<T>
}

impl<'a, T: 'a> serde::ser::Serialize for HateoasCollectionLinks<'a, T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_key("self")?;
        map.serialize_value(&HateoasLink {
            href: format!("{}?_cq={}&limit={}", self.p.endpoint, self.p.curr_token, self.p.limit),
            title: "songs",
        })?;
        map.serialize_key("prev")?;
        map.serialize_value(&HateoasLink {
            href: format!("{}?_cq={}&limit={}", self.p.endpoint, self.p.prev_token, self.p.limit),
            title: "Previous page",
        })?;
        map.serialize_key("next")?;
        map.serialize_value(&HateoasLink {
            href: format!("{}?_cq={}&limit={}", self.p.endpoint, self.p.next_token, self.p.limit),
            title: "Next page",
        })?;
        map.end()
    }
}

pub struct HateoasLink<'a>
{
    pub href: String,
    pub title: &'a str,
}

impl<'a> serde::ser::Serialize for HateoasLink<'a>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_key("href")?;
        map.serialize_value(&self.href)?;
        map.serialize_key("title")?;
        map.serialize_value(self.title)?;
        map.end()
    }
}