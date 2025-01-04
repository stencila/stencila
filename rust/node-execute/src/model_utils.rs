use codecs::{DecodeOptions, Format};
use common::{
    eyre::{bail, Result},
    tracing,
};
use models::{ModelOutput, ModelOutputKind, ModelTask};
use schema::{
    authorship, shortcuts::p, Article, AudioObject, AuthorRole, Block, ImageObject, Inline, Link,
    Node, VideoObject,
};

/// Performs a model task, converts the output to blocks, and
/// applies model authorship to those blocks.
///
/// Returns the block and the list of author roles.
#[tracing::instrument(skip_all)]
pub(super) async fn model_task_to_blocks_with_authors(
    task: ModelTask,
) -> Result<(Vec<Block>, Vec<AuthorRole>)> {
    let ModelOutput {
        authors,
        kind,
        format,
        content,
    } = models::perform_task(task).await?;

    let mut blocks = match kind {
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

    // Apply model authorship to blocks
    authorship(&mut blocks, authors.clone())?;

    Ok((blocks, authors))
}
