use codec_markdown_trait::to_markdown;
use codecs::{DecodeOptions, Format};
use common::{
    eyre::{bail, Result},
    tracing,
};
use models::{ModelOutput, ModelOutputKind, ModelTask};
use schema::{
    shortcuts::p, Article, AudioObject, AuthorRole, Block, File, ImageObject, Inline, Link,
    MessagePart, Node, Text, VideoObject,
};

/// Performs a model task and converts the output to blocks
///
/// Returns the block and the list of author roles.
#[tracing::instrument(skip_all)]
pub(super) async fn model_task_to_blocks_and_authors(
    task: ModelTask,
) -> Result<(Vec<Block>, Vec<AuthorRole>)> {
    let ModelOutput {
        authors,
        kind,
        format,
        content,
    } = models::perform_task(task).await?;

    let blocks = match kind {
        ModelOutputKind::Text => {
            // Decode the model output into blocks
            let node = codecs::from_str(
                &content,
                Some(DecodeOptions {
                    format: format
                        .is_unknown()
                        .then_some(Format::Markdown)
                        .or(Some(format)),
                    ..Default::default()
                }),
            )
            .await?;

            let Node::Article(Article { content, .. }) = node else {
                bail!("Expected content to be decoded to an article")
            };

            content
        }
        ModelOutputKind::Url => {
            let content_url = content;
            let media_type = Some(format.media_type());

            let node = if format.is_audio() {
                Inline::AudioObject(AudioObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else if format.is_image() {
                Inline::ImageObject(ImageObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else if format.is_video() {
                Inline::VideoObject(VideoObject {
                    content_url,
                    media_type,
                    ..Default::default()
                })
            } else {
                Inline::Link(Link {
                    target: content_url,
                    ..Default::default()
                })
            };

            vec![p([node])]
        }
    };

    Ok((blocks, authors))
}

/// Convert Stencila [`Block`] nodes to a [`MessagePart`]
pub(super) fn blocks_to_message_part(blocks: &Vec<Block>) -> Option<MessagePart> {
    let md = to_markdown(blocks);
    (!md.trim().is_empty()).then_some(MessagePart::Text(md.into()))
}

/// Convert a Stencila [`File`] to a [`MessagePart`]
pub(super) fn file_to_message_part(file: &File) -> Option<MessagePart> {
    let format = file
        .media_type
        .as_ref()
        .and_then(|media_type| Format::from_media_type(media_type).ok())
        .unwrap_or_else(|| Format::from_name(&file.name));

    if format.is_image() || format.is_audio() || format.is_video() {
        let Some(content) = file
            .content
            .as_ref()
            .and_then(|content| (!content.trim().is_empty()).then_some(content))
        else {
            return None;
        };

        let mut content_url = content.clone();
        let media_type = file.media_type.clone();

        if file.options.transfer_encoding.as_deref() == Some("base64")
            && !content_url.starts_with("data:")
        {
            let media_type = media_type.clone().unwrap_or_else(|| format.media_type());
            content_url.insert_str(0, &["data:", &media_type, ";base64,"].concat());
        }

        let object = if format.is_audio() {
            MessagePart::AudioObject(AudioObject {
                media_type,
                content_url,
                ..Default::default()
            })
        } else if format.is_video() {
            MessagePart::VideoObject(VideoObject {
                media_type,
                content_url,
                ..Default::default()
            })
        } else {
            MessagePart::ImageObject(ImageObject {
                media_type,
                content_url,
                ..Default::default()
            })
        };

        Some(object)
    } else {
        file.content
            .as_ref()
            .and_then(|content| (!content.trim().is_empty()).then_some(content))
            .map(|content| MessagePart::Text(Text::from(content)))
    }
}
