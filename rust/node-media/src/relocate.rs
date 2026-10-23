use std::path::{Path, PathBuf};

use eyre::Result;
use pathdiff::diff_paths;
use stencila_schema::{
    Block, CreativeWorkVariant, ImageObject, Inline, Node, VisitorMut, WalkControl, WalkNode,
};

use crate::naming::{MediaNamer, hash_bytes};

/// Rewrite local media and supplement references relative to a destination
/// document. Optionally copy the referenced originals, without decoding or
/// resizing them. Missing files and non-file URLs are left unchanged.
pub fn relocate_media<T: WalkNode>(
    node: &mut T,
    from_path: Option<&Path>,
    to_path: Option<&Path>,
    media_dir: Option<&Path>,
    supplements_dir: Option<&Path>,
) -> Result<()> {
    let from_dir = document_dir(from_path)?;
    let to_dir = document_dir(to_path)?;
    let mut visitor = Relocator {
        from_dir,
        to_dir,
        media_dir: media_dir.map(resolve).transpose()?,
        supplements_dir: supplements_dir.map(resolve).transpose()?,
        namer: MediaNamer::with_hashed_readable_names(),
        error: None,
    };
    visitor.walk(node);
    match visitor.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// A directory beside a file for retaining its referenced originals
///
/// Appends the suffix to the whole file name (e.g. `article.meca.media`)
/// rather than replacing the extension, so that it does not coincide with the
/// default media directory of an output beside it (e.g. `article.media` for
/// `article.md`), which encoders may remove before writing.
pub fn side_dir(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    name.into()
}

/// The directory of a document, with symlinks resolved
///
/// Decoders resolve references relative to the target of a symlinked
/// document (e.g. JATS canonicalizes its path), so the same base is used here.
fn document_dir(path: Option<&Path>) -> Result<PathBuf> {
    match path {
        Some(path) => {
            let path = resolve(path)?;
            Ok(path.parent().unwrap_or(Path::new("/")).into())
        }
        None => resolve(&std::env::current_dir()?),
    }
}

/// Make a path absolute, resolving symlinks in the part of it that exists
///
/// Paths that do not exist yet (e.g. an output file or directory) are resolved
/// via their closest existing ancestor so that all paths share the same form
/// when computing relative references between them.
fn resolve(path: &Path) -> Result<PathBuf> {
    let path = std::path::absolute(path)?;
    let mut existing = path.as_path();
    let mut rest = Vec::new();
    loop {
        if let Ok(canonical) = existing.canonicalize() {
            return Ok(rest
                .into_iter()
                .rev()
                .fold(canonical, |path, name| path.join(name)));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name);
                existing = parent;
            }
            _ => return Ok(path),
        }
    }
}

struct Relocator {
    from_dir: PathBuf,
    to_dir: PathBuf,
    media_dir: Option<PathBuf>,
    supplements_dir: Option<PathBuf>,
    namer: MediaNamer,
    error: Option<eyre::Report>,
}

enum Resource {
    Media,
    Supplement,
}

macro_rules! relocate_variants {
    ($self:ident, $value:ident, $typ:ident) => {
        match $value {
            $typ::ImageObject(media) => $self.relocate(&mut media.content_url, Resource::Media),
            $typ::AudioObject(media) => $self.relocate(&mut media.content_url, Resource::Media),
            $typ::VideoObject(media) => $self.relocate(&mut media.content_url, Resource::Media),
            _ => {}
        }
    };
}

impl Relocator {
    fn images(&mut self, images: &mut Option<Vec<ImageObject>>) {
        if let Some(images) = images {
            for image in images {
                self.relocate(&mut image.content_url, Resource::Media);
            }
        }
    }

    /// Walk the parts of a work (e.g. graphical abstracts, sub-articles)
    ///
    /// `parts` is not walked by `WalkNode`, so it is walked explicitly.
    fn parts(&mut self, parts: &mut Option<Vec<CreativeWorkVariant>>) {
        for part in parts.iter_mut().flatten() {
            self.walk(part);
        }
    }

    fn relocate(&mut self, url: &mut String, resource: Resource) {
        if self.error.is_some() {
            return;
        }
        if let Err(error) = self.rewrite(url, resource) {
            self.error = Some(error);
        }
    }

    fn rewrite(&mut self, url: &mut String, resource: Resource) -> Result<()> {
        // URLs with a scheme are not checked for because they can not be
        // distinguished from local names containing colons (e.g. `fig:1.png`)
        // or Windows paths (e.g. `C:\fig.png`). Only existing files are relocated.
        if url.is_empty()
            || url.starts_with('#')
            || url.starts_with("//")
            || url.starts_with("data:")
        {
            return Ok(());
        }
        let source = self.from_dir.join(&*url);
        if !source.is_file() {
            return Ok(());
        }
        let directory = match resource {
            Resource::Media => &self.media_dir,
            Resource::Supplement => &self.supplements_dir,
        };
        let target = if let Some(directory) = directory {
            let bytes = std::fs::read(&source)?;
            self.namer.write_bytes(
                directory,
                source.file_stem().and_then(|stem| stem.to_str()),
                source
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("bin"),
                hash_bytes(&bytes),
                &bytes,
            )?
        } else {
            source
        };
        *url = diff_paths(&target, &self.to_dir)
            .unwrap_or(target)
            .to_string_lossy()
            .into();
        Ok(())
    }
}

impl VisitorMut for Relocator {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        relocate_variants!(self, node, Node);
        match node {
            Node::MediaObject(media) => self.relocate(&mut media.content_url, Resource::Media),
            Node::MathBlock(math) => self.images(&mut math.options.images),
            Node::MathInline(math) => self.images(&mut math.options.images),
            Node::Table(table) => self.images(&mut table.options.images),
            Node::Article(article) => self.parts(&mut article.options.parts),
            Node::Supplement(supplement) => {
                if supplement.options.work.is_none()
                    && let Some(target) = &mut supplement.target
                {
                    self.relocate(target, Resource::Supplement);
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        relocate_variants!(self, block, Block);
        match block {
            Block::MathBlock(math) => self.images(&mut math.options.images),
            Block::Table(table) => self.images(&mut table.options.images),
            Block::Supplement(supplement) => {
                if supplement.options.work.is_none()
                    && let Some(target) = &mut supplement.target
                {
                    self.relocate(target, Resource::Supplement);
                }
            }
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkVariant) -> WalkControl {
        relocate_variants!(self, work, CreativeWorkVariant);
        match work {
            CreativeWorkVariant::MediaObject(media) => {
                self.relocate(&mut media.content_url, Resource::Media)
            }
            CreativeWorkVariant::Table(table) => self.images(&mut table.options.images),
            CreativeWorkVariant::Article(article) => self.parts(&mut article.options.parts),
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        relocate_variants!(self, inline, Inline);
        if let Inline::MediaObject(media) = inline {
            self.relocate(&mut media.content_url, Resource::Media);
        }
        if let Inline::MathInline(math) = inline {
            self.images(&mut math.options.images);
        }
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{Article, CreativeWorkVariant, ImageObject, Supplement};

    fn image_urls(node: &Node) -> Result<Vec<String>> {
        let json = serde_json::to_value(node)?;
        let mut urls = Vec::new();
        let mut stack = vec![&json];
        while let Some(value) = stack.pop() {
            match value {
                serde_json::Value::Object(object) => {
                    if let Some(url) = object.get("contentUrl").and_then(|url| url.as_str()) {
                        urls.push(url.to_string());
                    }
                    stack.extend(object.values());
                }
                serde_json::Value::Array(array) => stack.extend(array),
                _ => {}
            }
        }
        urls.sort();
        Ok(urls)
    }

    #[test]
    fn copies_originals_and_preserves_non_file_and_missing_references() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let from = dir.path().join("input/article.xml");
        let to = dir.path().join("output/article.json");
        std::fs::create_dir_all(from.parent().unwrap_or(dir.path()))?;
        std::fs::write(from.with_file_name("image.tif"), b"original bytes")?;
        std::fs::write(from.with_file_name("data.csv"), b"x\n1\n")?;
        let mut node = Node::Article(Article::new(vec![
            Block::ImageObject(ImageObject::new("image.tif".into())),
            Block::ImageObject(ImageObject::new("missing.png".into())),
            Block::ImageObject(ImageObject::new("https://example.org/image.png".into())),
            Block::Supplement(Supplement {
                target: Some("data.csv".into()),
                ..Default::default()
            }),
        ]));
        relocate_media(
            &mut node,
            Some(&from),
            Some(&to),
            Some(&dir.path().join("media")),
            Some(&dir.path().join("supplements")),
        )?;
        let json = serde_json::to_string(&node)?;
        assert!(json.contains("../media/image-"));
        assert!(json.contains("missing.png"));
        assert!(json.contains("https://example.org/image.png"));
        assert!(json.contains("../supplements/data-"));
        let image = std::fs::read_dir(dir.path().join("media"))?
            .next()
            .ok_or_else(|| eyre::eyre!("missing copy"))??;
        assert_eq!(std::fs::read(image.path())?, b"original bytes");
        Ok(())
    }

    #[test]
    fn copy_failures_are_returned_to_the_caller() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("article.xml");
        std::fs::write(dir.path().join("image.png"), b"bytes")?;
        let bad_dir = dir.path().join("not-a-directory");
        std::fs::write(&bad_dir, "file")?;
        let mut node = Node::ImageObject(ImageObject::new("image.png".into()));
        assert!(relocate_media(&mut node, Some(&path), None, Some(&bad_dir), None).is_err());
        Ok(())
    }

    #[test]
    fn relocates_media_in_nested_parts() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let from = dir.path().join("input/article.xml");
        let to = dir.path().join("output/article.json");
        std::fs::create_dir_all(dir.path().join("input"))?;
        std::fs::write(dir.path().join("input/abstract.png"), b"a")?;
        std::fs::write(dir.path().join("input/nested.png"), b"b")?;
        let mut sub_article = Article::new(vec![Block::ImageObject(ImageObject::new(
            "nested.png".into(),
        ))]);
        sub_article.options.parts = Some(vec![CreativeWorkVariant::ImageObject(ImageObject::new(
            "abstract.png".into(),
        ))]);
        let mut article = Article::new(Vec::new());
        article.options.parts = Some(vec![
            CreativeWorkVariant::ImageObject(ImageObject::new("abstract.png".into())),
            CreativeWorkVariant::Article(sub_article),
        ]);
        let mut node = Node::Article(article);
        relocate_media(&mut node, Some(&from), Some(&to), None, None)?;
        assert_eq!(
            image_urls(&node)?,
            vec![
                "../input/abstract.png",
                "../input/abstract.png",
                "../input/nested.png"
            ]
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn resolves_references_relative_to_symlink_target() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::create_dir_all(dir.path().join("target"))?;
        std::fs::create_dir_all(dir.path().join("link"))?;
        std::fs::write(dir.path().join("target/article.xml"), "<article/>")?;
        std::fs::write(dir.path().join("target/figure.png"), b"bytes")?;
        let from = dir.path().join("link/article.xml");
        std::os::unix::fs::symlink(dir.path().join("target/article.xml"), &from)?;
        let mut node = Node::ImageObject(ImageObject::new("figure.png".into()));
        relocate_media(
            &mut node,
            Some(&from),
            Some(&dir.path().join("link/out.json")),
            None,
            None,
        )?;
        assert_eq!(image_urls(&node)?, vec!["../target/figure.png"]);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn relocates_local_names_containing_colons() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let from = dir.path().join("input/article.xml");
        std::fs::create_dir_all(dir.path().join("input"))?;
        std::fs::write(dir.path().join("input/fig:1.png"), b"bytes")?;
        let mut node = Node::ImageObject(ImageObject::new("fig:1.png".into()));
        relocate_media(
            &mut node,
            Some(&from),
            Some(&dir.path().join("out.json")),
            None,
            None,
        )?;
        assert_eq!(image_urls(&node)?, vec!["input/fig:1.png"]);
        Ok(())
    }
}
