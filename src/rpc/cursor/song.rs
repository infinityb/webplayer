use serde;

use super::super::hateoas::HateoasLink;
use model::Song;

pub struct SongCursorResponse {
    pub next_token: String,
    pub curr_token: String,
    pub prev_token: String,
    pub limit: u32,
    pub items: Vec<Song>,
}

impl serde::ser::Serialize for SongCursorResponse
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        song_cursor_hateoas_partial(&mut map, self)?;
        map.serialize_key("_items")?;
        map.serialize_value(&self.items)?;
        map.end()
    }
}

fn song_cursor_hateoas_partial<S>(mm: &mut S, scr: &SongCursorResponse)
    -> Result<(), S::Error>
    where S: serde::ser::SerializeMap
{
    struct HateoasMeta<'a>
    {
        scr: &'a SongCursorResponse,
    }

    impl<'a> serde::ser::Serialize for HateoasMeta<'a>
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer
        {
            use serde::ser::SerializeMap;

            let mut map = serializer.serialize_map(None)?;

            map.serialize_key("self")?;
            map.serialize_value(&HateoasLink {
                href: format!("/songs?_cq={}&limit={}", self.scr.curr_token, self.scr.limit),
                title: "songs",
            })?;
            map.serialize_key("prev")?;
            map.serialize_value(&HateoasLink {
                href: format!("/songs?_cq={}&limit={}", self.scr.prev_token, self.scr.limit),
                title: "Previous page",
            })?;
            map.serialize_key("next")?;
            map.serialize_value(&HateoasLink {
                href: format!("/songs?_cq={}&limit={}", self.scr.next_token, self.scr.limit),
                title: "Next page",
            })?;
            map.end()
        }
    }

    mm.serialize_key("_links")?;
    mm.serialize_value(&HateoasMeta { scr: scr })?;
    Ok(())
}