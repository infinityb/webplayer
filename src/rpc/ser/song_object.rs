use serde;
use serde_json;

use super::to_json_value;
use super::super::hateoas::{self, HateoasObject, HateoasLink};
use model;


#[derive(Debug, Clone)]
pub struct SongObject<'a> {
    pub wrapped: &'a model::Song,
}

impl<'a> serde::ser::Serialize for SongObject<'a>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        song_hateoas_partial(&mut map, &self.wrapped)?;
        song_serialize_partial(&mut map, &self.wrapped, SongObjectOptions {
            include_blob: false,
            include_album: false,
            include_metadata: false,
            ..SongObjectOptions::default()
        })?;
        song_merged_metadata_partial(&mut map, &self.wrapped)?;
        map.end()
    }
}

fn song_merged_metadata_partial<S>(mm: &mut S, song: &model::Song)
    -> Result<(), S::Error>
    where S: serde::ser::SerializeMap
{
    struct SongMeta<'a>
    {
        song: &'a model::Song,
    }

    impl<'a> serde::ser::Serialize for SongMeta<'a>
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer
        {
            use serde::ser::SerializeMap;

            let length = self.song.metadata.len() + self.song.album.metadata.len();
            let mut map = serializer.serialize_map(Some(length))?;
            
            // FIXME(sell): we could end up with duplicate keys
            for (k, v) in self.song.metadata.iter()
            {
                map.serialize_key(&k.to_lowercase())?;
                map.serialize_value(v)?;
            }
            for (k, v) in self.song.album.metadata.iter()
            {
                map.serialize_key(&k.to_lowercase())?;
                map.serialize_value(v)?;
            }
            map.end()
        }
    }

    struct SongMetaInherited<'a>
    {
        song: &'a model::Song,
    }

    impl<'a> serde::ser::Serialize for SongMetaInherited<'a>
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer
        {
            use serde::ser::SerializeSeq;

            let length = self.song.album.metadata.len();
            let mut seq = serializer.serialize_seq(Some(length))?;
            
            // FIXME(sell): depending on how we fix the duplicate key
            // fixme above, we might need to match the behaviour here.
            for (k, v) in self.song.album.metadata.iter()
            {
                seq.serialize_element(&k.to_lowercase())?;
            }
            seq.end()
        }
    }

    mm.serialize_key("inherited_metadata")?;
    mm.serialize_value(&SongMetaInherited { song: song })?;
    mm.serialize_key("metadata")?;
    mm.serialize_value(&SongMeta { song: song })?;
    
    Ok(())
}

fn song_hateoas_partial<S>(mm: &mut S, song: &model::Song)
    -> Result<(), S::Error>
    where S: serde::ser::SerializeMap
{
    struct SongHateoasMeta<'a>
    {
        song: &'a model::Song,
    }

    // this is getting ugly - we need helper utilities.
    impl<'a> serde::ser::Serialize for SongHateoasMeta<'a>
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer
        {
            use serde::ser::SerializeMap;

            let mut map = serializer.serialize_map(None)?;

            map.serialize_key("self")?;
            map.serialize_value(&HateoasLink {
                href: format!("/song/{}", self.song.id.0),
                title: "song",
            })?;

            if let Some(ref art_blob) = self.song.album.art_blob {
                map.serialize_key("album_art")?;
                map.serialize_value(&HateoasLink {
                    href: format!("/blob/{}", art_blob),
                    title: "album_art",
                })?;
            }
            map.serialize_key("media")?;
            map.serialize_value(&HateoasLink {
                href: format!("/blob/{}", self.song.blob),
                title: "media",
            })?;

            // FIXME(sell): case insensitive key lookup?
            // FIXME(sell): query encoding?
            let mut artist: Option<&String> = None;
            if let Some(value) = self.song.album.metadata.get("ARTIST") {
                artist = Some(value);
            }
            if let Some(value) = self.song.metadata.get("ARTIST") {
                artist = Some(value);
            }
            if let Some(v) = artist {
                map.serialize_key("__experimental-artist")?;
                map.serialize_value(&HateoasLink {
                    href: format!("/artist/search?name={}", v),
                    title: "artist",
                })?;
            }

            map.serialize_key("album")?;
            map.serialize_value(&HateoasLink {
                href: format!("/album/{}", self.song.album.id.0),
                title: "album",
            })?;

            map.end()
        }
    }

    mm.serialize_key("_links")?;
    mm.serialize_value(&SongHateoasMeta { song: song })?;
    Ok(())
}

pub struct SongObjectOptions
{
    include_blob: bool,
    include_album: bool,
    include_metadata: bool,
}

impl Default for SongObjectOptions
{
    fn default() -> SongObjectOptions {
        SongObjectOptions {
            include_blob: true,
            include_album: true,
            include_metadata: true,
        }
    }
}

fn song_serialize_partial<S>(mm: &mut S, song: &model::Song, opts: SongObjectOptions)
    -> Result<(), S::Error>
    where S: serde::ser::SerializeMap
{
    use serde::ser::SerializeMap;

    mm.serialize_key("id")?;
    mm.serialize_value(&song.id.0)?;
    if opts.include_blob {
        mm.serialize_key("blob")?;
        mm.serialize_value(&song.blob)?;
    }
    mm.serialize_key("length_ms")?;
    mm.serialize_value(&song.length_ms)?;
    // mm.serialize_key("track_no")?;
    // mm.serialize_value(&song.track_no)?;
    if opts.include_metadata {
        mm.serialize_key("metadata")?;
        mm.serialize_value(&song.metadata)?;
    }
    if opts.include_album {
        mm.serialize_key("album")?;
        mm.serialize_value(&song.album)?;
    }

    Ok(())
}
