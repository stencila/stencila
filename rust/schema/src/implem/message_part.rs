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
        context.enter_elem("stencila-message-part");

        match self {
            MessagePart::Text(text) => {
                context
                    .push_attr("type", "text")
                    .push_text(&text.value.string);
            }
            MessagePart::ImageObject(image) => {
                context
                    .push_attr("type", "image")
                    .enter_elem("img")
                    .push_attr("src", &image.content_url)
                    .exit_elem();
            }
            MessagePart::AudioObject(audio) => {
                context
                    .push_attr("type", "audio")
                    .enter_elem("audio")
                    .push_attr("src", &audio.content_url)
                    .exit_elem();
            }
            MessagePart::VideoObject(video) => {
                context
                    .push_attr("type", "audio")
                    .enter_elem("video")
                    .push_attr("src", &video.content_url)
                    .exit_elem();
            }
        }

        context.exit_elem();
    }
}
