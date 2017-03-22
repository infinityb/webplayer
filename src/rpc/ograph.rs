use std::collections::{VecDeque, BTreeMap};

use serde;
use serde::ser::Error as SError;
use serde_json;

trait HateoasObject
{
    fn relative_url(&self) -> String;

    fn serialize(&self) -> serde_json::Result<serde_json::Value>;

    fn dependencies(&self) -> Vec<Box<HateoasObject>>;

    fn rels(&self) -> Vec<(String, String)>;
}

struct HateoasCollection<T>
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

struct HateoasLink<'a>
{
    href: String,
    title: &'a str,
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


fn into_json_value<S: ?Sized>(s: &S)
    -> serde_json::Result<serde_json::Value>
    where S: serde::ser::Serialize
{
    // this is horrible, sorry.
    // perhaps we could serialize into a flatbuffer that serializes into JSON
    let bytes = serde_json::to_vec(s)?;
    serde_json::from_slice(&bytes)
}

// mod test {

//     #[derive(Clone)]
//     struct Person {
//         id: i32,
//         name: String,
//         mother: Option<Box<Person>>,
//         father: Option<Box<Person>>,
//         spouse: Option<Box<Person>>,
//     }

//     struct PersonHateoas<'a>(&'a Person);
//     struct Ref { val: String }

//     impl serde::ser::Serialize for Ref
//     {
//         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//             where S: serde::Serializer
//         {
//             use serde::ser::SerializeMap;

//             let mut map = serializer.serialize_map(Some(1))?;
//             map.serialize_key("$ref")?;
//             map.serialize_value(&self.val)?;
//             map.end()
//         }
//     }

//     impl<'a> serde::ser::Serialize for PersonHateoas<'a>
//     {
//         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//             where S: serde::Serializer
//         {
//             use serde::ser::SerializeMap;

//             let mut map = serializer.serialize_map(None)?;
//             map.serialize_key("id")?;
//             map.serialize_value(&self.0.id)?;
//             map.serialize_key("name")?;
//             map.serialize_value(&self.0.name)?;
//             if let Some(ref mother) = self.0.mother {
//                 map.serialize_key("mother")?;
//                 map.serialize_value(&Ref { val: mother.relative_url() })?;
//             }
//             if let Some(ref father) = self.0.father {
//                 map.serialize_key("father")?;
//                 map.serialize_value(&Ref { val: father.relative_url() })?;
//             }
//             if let Some(ref spouse) = self.0.spouse {
//                 map.serialize_key("spouse")?;
//                 map.serialize_value(&Ref { val: spouse.relative_url() })?;
//             }
//             map.end()
//         }
//     }

//     impl HateoasObject for Person {
//         fn relative_url(&self) -> String
//         {
//             format!("/person/{}", self.id)
//         }

//         fn serialize(&self) -> serde_json::Result<serde_json::Value>
//         {
//             into_json_value(&PersonHateoas(self))
//         }

//         fn dependencies(&self) -> Vec<Box<HateoasObject>>
//         {
//             let mut out: Vec<Box<HateoasObject>> = Vec::new();
//             if let Some(ref mother) = self.mother {
//                 out.push(Box::new(*mother.clone()))
//             }
//             if let Some(ref father) = self.father {
//                 out.push(Box::new(*father.clone()))
//             }
//             if let Some(ref spouse) = self.spouse {
//                 out.push(Box::new(*spouse.clone()))
//             }
//             out
//         }
//     }
// }