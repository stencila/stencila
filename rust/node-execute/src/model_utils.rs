use codec_markdown::to_markdown_flavor;
use codecs::{DecodeOptions, Format};
use common::{
    eyre::{bail, Result},
    tracing,
};
use models::{ModelOutput, ModelOutputKind, ModelTask};
use schema::{
    shortcuts::p, Article, AudioObject, AuthorRole, Block, File, ImageObject, Inline,
    InstructionMessage, Link, MessagePart, MessageRole, Node, Text, VideoObject,
};

/// Render Stencila [`Block`] nodes to a "system prompt"
///
/// Uses a [`MarkdownEncodeContext`] with the render option set to true.
/// Used for generating Markdown from an executed prompt.
pub(super) fn blocks_to_system_message(blocks: &Vec<Block>) -> InstructionMessage {
    let md = to_markdown_flavor(blocks, Format::Llmd);

    InstructionMessage {
        role: Some(MessageRole::System),
        parts: vec![MessagePart::from(md)],
        ..Default::default()
    }
}

/// Convert Stencila [`Block`] nodes to a [`MessagePart`]
pub(super) fn blocks_to_message_part(blocks: &Vec<Block>) -> Option<MessagePart> {
    let md = to_markdown_flavor(blocks, Format::Llmd);

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
        let content = file
            .content
            .as_ref()
            .and_then(|content| (!content.trim().is_empty()).then_some(content))?;

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
            let format = format
                .is_unknown()
                .then_some(Format::Markdown)
                .or(Some(format));

            // Put thinking in an admonition if necessary
            // Some models (e.g. cloudflare/deepseek-r1-distill-qwen-32b) do not always produce the opening tag
            // so we only check for closing tag and assume everything above that is CoT tokens
            let content = if matches!(format, Some(Format::Markdown))
                && (content.contains("</think>\n") || content.contains("</thinking>\n"))
            {
                thinking_admonition(&content)
            } else {
                content
            };

            // Decode the model output into blocks
            let node = codecs::from_str(
                &content,
                Some(DecodeOptions {
                    format,
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

/// Put any chain-of-thought output into a collapsed admonition
fn thinking_admonition(content: &str) -> String {
    // Add admonition header and then prefix all lines until we reach the closing tag
    // Note that this assumes that everything thing before closing tag is thinking
    let mut lines = content.lines();
    let mut content = String::from("> [!info]+ Thinking\n>\n");
    for line in lines.by_ref() {
        if line == "<think>" || line == "<thinking>" {
            // Skip any opening tag
            continue;
        } else if line == "</think>" || line == "</thinking>" {
            // Separating blank line
            content.push('\n');
            break;
        }

        content.push_str("> ");
        content.push_str(line);
        content.push('\n');
    }

    // Add remainder of lines
    for line in lines {
        content.push_str(line);
        content.push('\n');
    }

    content
}

#[cfg(test)]
mod tests {
    use common_dev::insta::assert_snapshot;

    use super::*;

    #[test]
    fn balanced_think() {
        assert_snapshot!(thinking_admonition(r#"<think>
Mmmm,...

Another mmm...
</think>

Answer
"#), @r#"
        > [!info]+ Thinking
        >
        > Mmmm,...
        > 
        > Another mmm...


        Answer
        "#);
    }

    #[test]
    fn balanced_thinking() {
        assert_snapshot!(thinking_admonition(r#"<thinking>
Mmmm,...

Another mmm...
</thinking>

Answer

Still going answer
"#), @r#"
        > [!info]+ Thinking
        >
        > Mmmm,...
        > 
        > Another mmm...


        Answer

        Still going answer
        "#);
    }

    #[test]
    fn unbalanced_think() {
        assert_snapshot!(thinking_admonition(r#"Mmmm,...

Another mmm...
</think>

Answer
"#), @r#"
        > [!info]+ Thinking
        >
        > Mmmm,...
        > 
        > Another mmm...


        Answer
        "#);
    }

    #[test]
    fn unbalanced_thinking() {
        assert_snapshot!(thinking_admonition(r#"Mmmm,...

Another mmm...
</thinking>

Answer

Still going answer
"#), @r#"
        > [!info]+ Thinking
        >
        > Mmmm,...
        > 
        > Another mmm...


        Answer

        Still going answer
        "#);
    }

    #[test]
    fn lists_in_thinking() {
        assert_snapshot!(thinking_admonition(r#"<think>
Para one

- apple
- pear

Para two

1. one
2. two

Para three
</think>

Answer
"#), @r#"
        > [!info]+ Thinking
        >
        > Para one
        > 
        > - apple
        > - pear
        > 
        > Para two
        > 
        > 1. one
        > 2. two
        > 
        > Para three


        Answer
        "#);
    }
}
