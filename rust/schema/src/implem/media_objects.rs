use codec_losses::lost_options;

use crate::{prelude::*, AudioObject, ImageObject, MediaObject, VideoObject};

macro_rules! html_attrs {
    ($object:expr) => {{
        use codec_html_trait::encode::attr;

        let mut attrs = vec![attr("src", $object.content_url.as_str())];

        if let Some(caption) = &$object.caption {
            attrs.push(attr("alt", &caption.to_text().0))
        }

        if let Some(title) = &$object.title {
            attrs.push(attr("title", &title.to_text().0))
        }

        attrs
    }};
}

macro_rules! jats_attrs {
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

macro_rules! jats_content {
    ($object:expr) => {{
        let mut content = String::new();

        if let Some(caption) = &$object.caption {
            use codec_jats_trait::encode::escape;

            let caption = caption.to_text().0;
            content.push_str(&["<alt-text>", &escape(caption), "</alt-text>"].concat())
        }

        content
    }};
}

macro_rules! to_markdown {
    ($object:expr, $context:expr) => {{
        let mut losses = lost_options!($object, id);

        let (caption_md, caption_losses) = $object.caption.to_markdown($context);
        losses.merge(caption_losses);

        let mut md = ["![", &caption_md, "](", &$object.content_url].concat();

        if let Some(title) = &$object.title {
            let (title_text, title_losses) = title.to_text();
            losses.merge(title_losses);

            md.push_str(" \"");
            md.push_str(&title_text);
            md.push('"');
        }

        md.push(')');

        (md, losses)
    }};
}

impl MediaObject {
    pub fn to_jats_special(&self) -> (String, Losses) {
        // It is necessary to have special JATS functions for these types
        // to split the `media_type` field into separate `mimetype` and `media-subtype`
        // attributes and to ensure `AudioObject` and `VideoObject` ad differentiated
        // through the `mimetype` attribute

        use codec_jats_trait::encode::elem;

        (elem("inline-media", jats_attrs!(self), ""), Losses::todo())
    }
}

impl AudioObject {
    pub fn to_html_special(&self, _context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::elem;

        let mut attrs = html_attrs!(self);
        attrs.push("controls".to_string());

        elem("audio", &attrs, &[])
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let mut attrs = jats_attrs!(self);
        if !attrs.iter().any(|(name, ..)| name == &"mimetype") {
            attrs.push(("mimetype", "audio"));
        }

        (
            elem("inline-media", attrs, jats_content!(self)),
            Losses::todo(),
        )
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let (md, mut losses) = to_markdown!(self, context);
        losses.merge(lost_options!(self.options, transcript));
        (md, losses)
    }
}

impl ImageObject {
    pub fn to_html_special(&self, _context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::elem;

        elem("img", &html_attrs!(self), &[])
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("inline-graphic", jats_attrs!(self), jats_content!(self)),
            Losses::todo(),
        )
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let (md, mut losses) = to_markdown!(self, context);
        losses.merge(lost_options!(self.options, thumbnail));
        (md, losses)
    }
}

impl VideoObject {
    pub fn to_html_special(&self, _context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::elem;

        let mut attrs = html_attrs!(self);
        attrs.push("controls".to_string());

        elem("video", &attrs, &[])
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let mut attrs = jats_attrs!(self);
        if !attrs.iter().any(|(name, ..)| name == &"mimetype") {
            attrs.push(("mimetype", "video"));
        }

        (
            elem("inline-media", attrs, jats_content!(self)),
            Losses::todo(),
        )
    }

    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let (md, mut losses) = to_markdown!(self, context);
        losses.merge(lost_options!(self.options, thumbnail, transcript));
        (md, losses)
    }
}
