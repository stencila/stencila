use crate::{prelude::*, AudioObject, ImageObject, MediaObject, VideoObject};

macro_rules! attrs {
    ($object:expr) => {{
        let mut attrs = vec![("xlink:href", $object.content_url.as_str())];

        if let Some(media_type) = &$object.media_type {
            let mut parts = media_type.split('/');
            if let Some(mime_type) = parts.next() {
                attrs.push(("mimetype", mime_type))
            }
            if let Some(mime_subtype) = parts.next() {
                attrs.push(("mime-subtype", mime_subtype))
            }
        }

        attrs
    }};
}

impl MediaObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        // It is necessary to have special JATS functions for these types
        // to split the `media_type` field into separate `mimetype` and `media-subtype`
        // attributes and to ensure `AudioObject` and `VideoObject` ad differentiated
        // through the `mimetype` attribute

        use codec_jats_trait::encode::elem;

        (elem("inline-media", attrs!(self), ""), Losses::todo())
    }
}

impl AudioObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let mut attrs = attrs!(self);
        if !attrs.iter().any(|(name, ..)| name == &"mimetype") {
            attrs.push(("mimetype", "audio"));
        }

        (elem("inline-media", attrs, ""), Losses::todo())
    }
}

impl ImageObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (elem("inline-graphic", attrs!(self), ""), Losses::todo())
    }
}

impl VideoObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let mut attrs = attrs!(self);
        if !attrs.iter().any(|(name, ..)| name == &"mimetype") {
            attrs.push(("mimetype", "video"));
        }

        (elem("inline-media", attrs, ""), Losses::todo())
    }
}
