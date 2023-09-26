use crate::{prelude::*, AudioObject, ImageObject, MediaObject, VideoObject};

impl MediaObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        // It is necessary to have special JATS functions for these types
        // to split the `media_type` field into separate `mediatype` and `media-subtype`
        // attributes

        use codec_jats_trait::encode::elem;

        (
            elem("inline-media", [("xlink:href", &self.content_url)], ""),
            Losses::todo(),
        )
    }
}

impl AudioObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("inline-media", [("xlink:href", &self.content_url)], ""),
            Losses::todo(),
        )
    }
}

impl ImageObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("inline-graphic", [("xlink:href", &self.content_url)], ""),
            Losses::todo(),
        )
    }
}

impl VideoObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("inline-media", [("xlink:href", &self.content_url)], ""),
            Losses::todo(),
        )
    }
}
