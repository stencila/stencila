use std::path::PathBuf;

use codec_info::lost_options;
use common::tracing;

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
    ($object:expr, $context:expr, $losses:expr) => {{
        $context
            .enter_node($object.node_type(), $object.node_id())
            .merge_losses(lost_options!($object, id))
            .merge_losses($losses)
            .push_str("![");

        if let Some(caption) = &$object.caption {
            $context.push_prop_fn(NodeProperty::Caption, |context| {
                caption.to_markdown(context)
            });
        }

        $context
            .push_str("](")
            .push_prop_str(NodeProperty::ContentUrl, &$object.content_url);

        if let Some(title) = &$object.title {
            $context
                .push_str(" \"")
                .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context))
                .push_str("\"");
        }

        $context.push_str(")").exit_node();
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
}

impl MarkdownCodec for AudioObject {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        to_markdown!(self, context, lost_options!(self.options, transcript))
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
}

impl DomCodec for ImageObject {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let mut img = true;
        if let Some(media_type) = &self.media_type {
            context.push_attr("media-type", &media_type);

            // For media types that require rendering in the browser, add `content_url` as an
            // attribute that is easily accessible by the <stencila-image-object> custom element
            // and do not add an <img> tag.
            if media_type == "text/vnd.mermaid" {
                context.push_attr("content", &self.content_url);
                img = false;
            }
        }

        if img {
            // If the document is being encoded standalone, and the image URL is data URI
            // or file system path (possibly with a relative URL) the ensure that we have
            // create an on disk copy
            if context.standalone
                && !(self.content_url.starts_with("http://")
                    || self.content_url.starts_with("https://"))
            {
                // Encode the data URI to a file
                let images_path = match context.to_path.as_deref() {
                    // images directory will be a sibling to the encoded file
                    Some(to_path) => {
                        PathBuf::from(to_path.to_string_lossy().to_string() + ".images")
                    }
                    // images directory will be in the current directory
                    None => PathBuf::from("images"),
                };

                let image_path = if self.content_url.starts_with("data:") {
                    match images::data_uri_to_path(&self.content_url, &images_path) {
                        Ok(path) => Some(path),
                        Err(error) => {
                            tracing::warn!("While encoding image data URI to file: {error}");
                            None
                        }
                    }
                } else {
                    match images::file_uri_to_path(
                        &self.content_url,
                        context.from_path.as_deref(),
                        &images_path,
                    ) {
                        Ok(path) => Some(path),
                        Err(error) => {
                            tracing::warn!("While encoding image to file: {error}");
                            None
                        }
                    }
                };

                let src = match image_path {
                    Some(image_path) => {
                        // Make the image path relative to the destination file
                        match context.to_path.as_deref() {
                            Some(to_path) => to_path
                                .parent()
                                .and_then(|dir| image_path.strip_prefix(dir).ok())
                                .map(PathBuf::from)
                                .unwrap_or(image_path),
                            None => image_path,
                        }
                        .to_string_lossy()
                        .to_string()
                    }
                    None => {
                        // Fallback to encoding the original URL
                        self.content_url.to_string()
                    }
                };

                context.enter_elem("img").push_attr("src", &src).exit_elem();

                // If the document image is not set yet, then set it
                if context.image().is_none() {
                    context.set_image(&src);
                }
            } else {
                // Not standalone, so just encode content URL as image (optimize for speed)
                context
                    .enter_elem("img")
                    .push_attr("src", &self.content_url)
                    .exit_elem();
            }
        }

        if let Some(title) = &self.title {
            context.push_slot_fn("span", "title", |context| title.to_dom(context));
        }

        if let Some(caption) = &self.caption {
            context.push_slot_fn("span", "caption", |context| caption.to_dom(context));
        }

        if let Some(authors) = &self.options.authors {
            context.push_slot_fn("span", "authors", |context| authors.to_dom(context));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for ImageObject {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        to_markdown!(self, context, lost_options!(self.options, thumbnail))
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
}

impl MarkdownCodec for VideoObject {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        to_markdown!(
            self,
            context,
            lost_options!(self.options, thumbnail, transcript)
        )
    }
}
