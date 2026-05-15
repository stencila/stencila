use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use eyre::Result;

use stencila_codec_info::EncodedAsset;
use stencila_schema::{
    AudioObject, Block, CodeChunk, CreativeWorkVariant, Figure, ImageObject, Inline, Node, NodeId,
    NodeType, VideoObject, VisitorMut, WalkControl, WalkNode,
};

use crate::naming::MediaNamer;

/// Collect local media file references without copying or rewriting URLs.
///
/// Each returned [`EncodedAsset`] points at an existing local file and is
/// annotated with the originating node's id/type and role. This is useful for
/// follow-up systems such as Content Credentials that need to reason about
/// static media referenced by an export, while leaving document content and
/// media bytes untouched.
pub fn reference_media_with_paths<T>(
    node: &T,
    document_path: Option<&Path>,
) -> Result<Vec<EncodedAsset>>
where
    T: WalkNode + Clone,
{
    let document_dir = match document_path {
        Some(path) => match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => PathBuf::from("."),
        },
        None => current_dir()?,
    };

    let mut walker = Referencer {
        document_dir,
        parent_stack: Vec::new(),
        namer: MediaNamer::new(),
        assets: Vec::new(),
    };
    let mut copy = node.clone();
    walker.walk(&mut copy);

    Ok(walker.assets)
}

struct Referencer {
    /// Directory used to resolve relative media paths.
    document_dir: PathBuf,

    /// Stack of ancestor structs, used to attribute static assets to the
    /// closest meaningful originating node.
    parent_stack: Vec<(NodeType, NodeId)>,

    /// State used to derive readable media titles from nearby node labels.
    namer: MediaNamer,

    /// The asset records produced during reference discovery.
    assets: Vec<EncodedAsset>,
}

impl Referencer {
    fn resolve_media(&self, content_url: &str) -> Option<PathBuf> {
        if !should_reference_url(content_url) {
            return None;
        }

        let path = Path::new(content_url);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.document_dir.join(path)
        };

        path.is_file().then_some(path)
    }

    fn record_asset(
        &mut self,
        path: PathBuf,
        self_id: Option<&NodeId>,
        self_type: NodeType,
        title: Option<String>,
    ) {
        let (node_type, node_id) = self.originating(self_id, self_type);
        let role = role_for(node_type);
        self.assets.push(EncodedAsset {
            path,
            node_id: node_id.map(|id| id.to_string()),
            node_type: Some(node_type.to_string()),
            role: Some(role.to_string()),
            title,
            ..Default::default()
        });
    }

    fn originating(
        &self,
        self_id: Option<&NodeId>,
        self_type: NodeType,
    ) -> (NodeType, Option<NodeId>) {
        if let Some((node_type, node_id)) = self
            .parent_stack
            .iter()
            .rev()
            .find(|(node_type, _)| is_executable(*node_type))
        {
            return (*node_type, Some(node_id.clone()));
        }

        if let Some((node_type, node_id)) = self
            .parent_stack
            .iter()
            .rev()
            .find(|(node_type, _)| is_media_container(*node_type))
        {
            return (*node_type, Some(node_id.clone()));
        }

        (self_type, self_id.cloned())
    }

    fn reference_image(&mut self, image: &mut ImageObject) {
        if image.is_viz() {
            return;
        }

        if let Some(path) = self.resolve_media(&image.content_url) {
            let id = image.node_id();
            let title = self.namer.next_media_title(image.title.as_deref());
            self.record_asset(path, Some(&id), NodeType::ImageObject, title);
        }
    }

    fn reference_images(&mut self, images: &mut [ImageObject]) {
        images
            .iter_mut()
            .for_each(|image| self.reference_image(image));
    }

    fn reference_audio(&mut self, audio: &mut AudioObject) {
        if let Some(path) = self.resolve_media(&audio.content_url) {
            let id = audio.node_id();
            let title = self.namer.next_media_title(audio.title.as_deref());
            self.record_asset(path, Some(&id), NodeType::AudioObject, title);
        }
    }

    fn reference_video(&mut self, video: &mut VideoObject) {
        if let Some(path) = self.resolve_media(&video.content_url) {
            let id = video.node_id();
            let title = self.namer.next_media_title(video.title.as_deref());
            self.record_asset(path, Some(&id), NodeType::VideoObject, title);
        }
    }

    fn reference_figure(&mut self, figure: &mut Figure) {
        self.parent_stack.push((NodeType::Figure, figure.node_id()));

        if let Some(caption) = &mut figure.caption {
            caption.walk_mut(self);
        }

        self.namer.push_figure(figure);
        figure.content.walk_mut(self);
        self.namer.pop();

        self.parent_stack.pop();
    }

    fn reference_code_chunk(&mut self, chunk: &mut CodeChunk) {
        self.parent_stack
            .push((NodeType::CodeChunk, chunk.node_id()));

        if let Some(caption) = &mut chunk.caption {
            caption.walk_mut(self);
        }

        self.namer.push_code_chunk(chunk);
        chunk.outputs.walk_mut(self);
        self.namer.pop();

        self.parent_stack.pop();
    }
}

fn should_reference_url(content_url: &str) -> bool {
    !content_url.starts_with("data:")
        && !content_url.starts_with("http://")
        && !content_url.starts_with("https://")
}

fn is_executable(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Button
            | NodeType::CallBlock
            | NodeType::CodeChunk
            | NodeType::CodeExpression
            | NodeType::ForBlock
            | NodeType::Form
            | NodeType::IfBlock
            | NodeType::IfBlockClause
            | NodeType::IncludeBlock
            | NodeType::InstructionBlock
            | NodeType::InstructionInline
            | NodeType::Parameter
            | NodeType::PromptBlock
    )
}

fn is_media_container(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::MathBlock | NodeType::MathInline | NodeType::Table | NodeType::Figure
    )
}

fn role_for(node_type: NodeType) -> &'static str {
    if is_executable(node_type) {
        return "computational-output";
    }
    match node_type {
        NodeType::MathBlock | NodeType::MathInline => "math-image",
        NodeType::Table => "table-image",
        NodeType::Figure => "figure",
        _ => "figure",
    }
}

impl VisitorMut for Referencer {
    fn enter_struct(&mut self, node_type: NodeType, node_id: NodeId) -> WalkControl {
        self.parent_stack.push((node_type, node_id));
        WalkControl::Continue
    }

    fn exit_struct(&mut self) {
        self.parent_stack.pop();
    }

    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::CodeChunk(chunk) => {
                self.reference_code_chunk(chunk);
                return WalkControl::Break;
            }
            Node::Figure(figure) => {
                self.reference_figure(figure);
                return WalkControl::Break;
            }
            Node::AudioObject(audio) => self.reference_audio(audio),
            Node::ImageObject(image) => self.reference_image(image),
            Node::VideoObject(video) => self.reference_video(video),
            Node::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.reference_images(images)
                }
            }
            Node::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.reference_images(images)
                }
            }
            Node::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.reference_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkVariant) -> WalkControl {
        match work {
            CreativeWorkVariant::AudioObject(audio) => self.reference_audio(audio),
            CreativeWorkVariant::Figure(figure) => {
                self.reference_figure(figure);
                return WalkControl::Break;
            }
            CreativeWorkVariant::ImageObject(image) => self.reference_image(image),
            CreativeWorkVariant::VideoObject(video) => self.reference_video(video),
            CreativeWorkVariant::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.reference_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::AudioObject(audio) => self.reference_audio(audio),
            Block::CodeChunk(chunk) => {
                self.reference_code_chunk(chunk);
                return WalkControl::Break;
            }
            Block::Figure(figure) => {
                self.reference_figure(figure);
                return WalkControl::Break;
            }
            Block::ImageObject(image) => self.reference_image(image),
            Block::VideoObject(video) => self.reference_video(video),
            Block::MathBlock(math) => {
                if let Some(images) = &mut math.options.images {
                    self.reference_images(images)
                }
            }
            Block::Table(table) => {
                if let Some(images) = &mut table.options.images {
                    self.reference_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::AudioObject(audio) => self.reference_audio(audio),
            Inline::ImageObject(image) => self.reference_image(image),
            Inline::VideoObject(video) => self.reference_video(video),
            Inline::MathInline(math) => {
                if let Some(images) = &mut math.options.images {
                    self.reference_images(images)
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use std::fs::write;

    use eyre::{OptionExt, Result, bail};
    use tempfile::tempdir;

    use stencila_schema::{Figure, Inline, Paragraph, Text};

    use super::*;

    #[test]
    fn references_static_figure_media_using_figure_attribution() -> Result<()> {
        let dir = tempdir()?;
        let document_path = dir.path().join("source.md");
        let image_path = dir.path().join("plot.png");
        write(&document_path, "")?;
        write(&image_path, [0u8])?;

        let block = Block::Figure(Figure {
            id: Some("Fig 1".to_string()),
            label: Some("1".to_string()),
            caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from("A figure caption."),
            )]))]),
            content: vec![Block::ImageObject(ImageObject::new("plot.png".to_string()))],
            ..Default::default()
        });

        let assets = reference_media_with_paths(&block, Some(&document_path))?;

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].path, image_path);
        assert_eq!(assets[0].node_type.as_deref(), Some("Figure"));
        assert_eq!(assets[0].role.as_deref(), Some("figure"));
        assert_eq!(
            assets[0].title.as_deref(),
            Some("Figure 1: A figure caption.")
        );

        let Block::Figure(figure) = block else {
            bail!("expected figure")
        };
        let Block::ImageObject(image) = figure.content.first().ok_or_eyre("image")? else {
            bail!("expected image")
        };
        assert_eq!(image.content_url, "plot.png");

        Ok(())
    }
}
