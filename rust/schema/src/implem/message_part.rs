use crate::{prelude::*, MessagePart, Text};

impl<S> From<S> for MessagePart
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::Text(Text::from(value))
    }
}

impl DomCodec for MessagePart {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        let (r#type, value) = match self {
            MessagePart::Text(text) => ("text", &text.value.string),
            MessagePart::ImageObject(image) => ("image", &image.content_url),
            MessagePart::AudioObject(audio) => ("audio", &audio.content_url),
            MessagePart::VideoObject(video) => ("video", &video.content_url),
        };

        context
            .enter_elem("stencila-message-part")
            .push_attr("type", r#type)
            .push_attr("value", value)
            .exit_elem();
    }
}
