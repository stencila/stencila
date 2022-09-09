use std::path::Path;

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
    },
    stencila_schema::{
        Article, AudioObject, Date, Directory, DirectoryParts, File, ImageObject, Node, Person,
        SoftwareSourceCode, VideoObject,
    },
    utils::{some_box_string, vec_string},
    Codec, CodecTrait, DecodeOptions,
};
use formats::{match_name, FormatNodeType};

/// A fallback codec that decodes a node based on the node type associated
/// with a format (for `from_str`), or based on the characteristics of the
/// path (for `from_path`).
pub struct FormatCodec;

#[async_trait]
impl CodecTrait for FormatCodec {
    fn spec() -> Codec {
        Codec {
            formats: vec_string!["*"],
            root_types: vec_string![
                "Article",
                "AudioObject",
                "ImageObject",
                "VideoObject",
                "SoftwareSourceCode",
                "Date",
                "Person",
                "Directory",
                "File"
            ],
            from_string: true,
            from_path: true,
            ..Default::default()
        }
    }

    fn from_str(content: &str, options: Option<DecodeOptions>) -> Result<Node> {
        let format_name = match options.unwrap_or_default().format {
            Some(format) => format,
            None => bail!("Must provide a format to be decoded"),
        };

        let format = match_name(&format_name).spec();

        let node = match format.node_type {
            FormatNodeType::Article => Node::Article(Article {
                text: some_box_string!(content),
                ..Default::default()
            }),
            FormatNodeType::AudioObject => Node::AudioObject(AudioObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::ImageObject => Node::ImageObject(ImageObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::VideoObject => Node::VideoObject(VideoObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::SoftwareSourceCode => Node::SoftwareSourceCode(SoftwareSourceCode {
                text: some_box_string!(content),
                programming_language: match format_name.is_empty() {
                    true => None,
                    false => some_box_string!(format_name),
                },
                ..Default::default()
            }),
            FormatNodeType::Date => Node::Date(Date {
                value: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::Person => Node::Person(Person {
                name: some_box_string!(content),
                ..Default::default()
            }),
            _ => bail!(
                "Unable to create node from a string of format `{}`",
                format.title
            ),
        };
        Ok(node)
    }

    async fn from_path(path: &Path, _options: Option<DecodeOptions>) -> Result<Node> {
        Ok(if path.exists() {
            if path.is_dir() {
                Node::Directory(directory_from_path(path))
            } else {
                Node::File(file_from_path(path))
            }
        } else if path.extension().is_some() {
            Node::File(file_from_path(path))
        } else {
            Node::Directory(directory_from_path(path))
        })
    }
}

/// Create a [`File`] node from a filesystem path
fn file_from_path<P: AsRef<Path>>(path: P) -> File {
    let path = path.as_ref();
    File {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        ..Default::default()
    }
}

/// Create a [`Directory`] node from a filesystem path
fn directory_from_path<P: AsRef<Path>>(path: P) -> Directory {
    let path = path.as_ref();
    Directory {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        parts: directory_parts(path),
        ..Default::default()
    }
}

/// Create a vector of [`DirectoryParts`] node from a filesystem path
fn directory_parts(path: &Path) -> Vec<DirectoryParts> {
    path.read_dir()
        .unwrap()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                DirectoryParts::Directory(directory_from_path(&path))
            } else {
                DirectoryParts::File(file_from_path(&path))
            }
        })
        .sorted_by(|a, b| {
            let a = match a {
                DirectoryParts::File(File { name, .. })
                | DirectoryParts::Directory(Directory { name, .. }) => name,
            };
            let b = match b {
                DirectoryParts::File(File { name, .. })
                | DirectoryParts::Directory(Directory { name, .. }) => name,
            };
            a.cmp(b)
        })
        .collect()
}
