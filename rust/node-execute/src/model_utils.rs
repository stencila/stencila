use codecs::{DecodeOptions, Format};
use common::{
    eyre::{bail, Result},
    tracing,
};
use models::{ModelOutput, ModelOutputKind, ModelTask};
use schema::{
    shortcuts::p, Article, AudioObject, AuthorRole, Block, File, ImageObject, Inline, Link, MessagePart, Node, Text, VideoObject
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

/// Convert a Stencila [`File`] to a [`MessagePart`]
pub(super) fn file_to_message_part(file: &File) -> Option<MessagePart> {
    let format = file
        .media_type
        .as_ref()
        .and_then(|media_type| Format::from_media_type(media_type).ok())
        .unwrap_or_else(|| Format::from_name(&file.name));

    if format.is_image() {
        None // TODO
    } else if format.is_audio() {
        None // TODO
    } else if format.is_video() {
        None // TODO
    } else {
        file.content
            .as_ref()
            .map(|value| MessagePart::Text(Text::from(value)))
    }
}
