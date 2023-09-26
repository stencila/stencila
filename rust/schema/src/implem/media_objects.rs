use crate::{prelude::*, AudioObject, ImageObject, MediaObject, VideoObject};

impl MediaObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "inline-media",
                [("xlink:href".to_string(), self.content_url.clone())],
                "",
            ),
            Losses::todo(),
        )
    }
}

impl AudioObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "inline-media",
                [("xlink:href".to_string(), self.content_url.clone())],
                "",
            ),
            Losses::todo(),
        )
    }
}

impl ImageObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "inline-graphic",
                [("xlink:href".to_string(), self.content_url.clone())],
                "",
            ),
            Losses::todo(),
        )
    }
}

impl VideoObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "inline-media",
                [("xlink:href".to_string(), self.content_url.clone())],
                "",
            ),
            Losses::todo(),
        )
    }
}
