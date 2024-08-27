use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use ignore::Walk;

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    format::Format,
    schema::{Directory, File, FileOrDirectory, Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions,
};

/// A codec for "decoding" a directory to a Stencila `Directory` node
pub struct DirectoryCodec;

#[async_trait]
impl Codec for DirectoryCodec {
    fn name(&self) -> &str {
        "directory"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Directory => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_from_bytes(&self) -> bool {
        false
    }

    fn supports_to_bytes(&self) -> bool {
        false
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    fn supports_from_path(&self) -> bool {
        true
    }

    fn supports_to_path(&self) -> bool {
        false
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        match node_type {
            NodeType::Directory => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    async fn from_path(
        &self,
        root: &Path,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let root = root.canonicalize()?;

        let mut dirs: Vec<(usize, Directory)> = Vec::new();

        // Sort parts of a directory as per usual, with subdirectories first
        fn dir_sort(a: &FileOrDirectory, b: &FileOrDirectory) -> Ordering {
            if let (
                FileOrDirectory::Directory(Directory { path: a, .. }),
                FileOrDirectory::Directory(Directory { path: b, .. }),
            ) = (a, b)
            {
                // Both directories so sort by path
                PathBuf::from(a).cmp(&PathBuf::from(b))
            } else if let (
                FileOrDirectory::File(File { path: a, .. }),
                FileOrDirectory::File(File { path: b, .. }),
            ) = (a, b)
            {
                // Both files so sort by path
                PathBuf::from(a).cmp(&PathBuf::from(b))
            } else if matches!(a, FileOrDirectory::Directory(..)) {
                // a is a directory, but b is not
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }

        // Fold the tail of directories into its parents
        fn dirs_fold(dirs: &mut Vec<(usize, Directory)>) {
            let (.., mut last) = dirs.pop().expect("dirs should not be empty");
            last.parts.sort_by(dir_sort);
            let (.., parent) = dirs.last_mut().expect("dirs should not be empty");
            parent.parts.push(FileOrDirectory::Directory(last))
        }

        for entry in Walk::new(&root).flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .map_or_else(String::new, |name| name.to_string_lossy().to_string());
            let depth = path.components().count();

            let path_string = path
                .strip_prefix(&root)
                .expect("should always be nested within root")
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
                let dir = Directory {
                    name,
                    path: path_string,
                    ..Default::default()
                };

                while dirs.len() > 1
                    && dirs
                        .last()
                        .map(|(last_depth, ..)| depth <= *last_depth)
                        .unwrap_or_default()
                {
                    dirs_fold(&mut dirs);
                }

                dirs.push((depth, dir))
            } else {
                let media_type = mime_guess::from_path(path)
                    .first()
                    .map(|mime| mime.to_string());

                let file = FileOrDirectory::File(File {
                    name,
                    path: path_string,
                    media_type,
                    ..Default::default()
                });

                while dirs.len() > 1
                    && dirs
                        .last()
                        .map(|(last_depth, ..)| (depth - 1) < *last_depth)
                        .unwrap_or_default()
                {
                    dirs_fold(&mut dirs);
                }

                if let Some((.., parent)) = dirs.last_mut() {
                    parent.parts.push(file);
                }
            }
        }

        while dirs.len() > 1 {
            dirs_fold(&mut dirs);
        }

        if dirs.is_empty() {
            bail!("No directories walked")
        }

        let mut root = dirs.swap_remove(0).1;
        root.parts.sort_by(dir_sort);

        let node = Node::Directory(root);

        Ok((node, DecodeInfo::none()))
    }
}
