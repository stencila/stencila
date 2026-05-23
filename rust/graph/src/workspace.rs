//! Workspace graph extraction.
//!
//! This module walks a directory tree, represents filesystem objects, and
//! optionally links decoded document graphs back to the files they came from.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::Metadata,
    path::{Path, PathBuf},
};

use eyre::{Result, WrapErr, ensure};
use ignore::WalkBuilder;
use stencila_codecs::{CodecDirection, DecodeOptions, Format};
use stencila_schema::{
    DateTime as SchemaDateTime, Directory, File, Graph, GraphEdgeKind, Node, SymbolicLink,
};

use crate::{
    DocumentReferenceKind, GraphBuilder, add_document_with_reference_resolver,
    code::{self, CodeLanguage},
    environment, evidence,
    ids::{LocalGraphId, WorkspaceRelPath},
    reference::{
        document_relative_workspace_path, is_local_relative_reference, normalize_path_lexically,
        reference_path_candidates,
    },
};

/// Options for building a graph from a workspace directory.
///
/// Keeping these options together makes workspace graph construction explicit
/// about decoding behavior and failure policy at the call site.
#[derive(Debug, Clone)]
pub struct WorkspaceOptions {
    /// Explicit graph subject. Defaults to `workspace:<root-name>`.
    ///
    /// Supplying a subject lets callers pin graph identity to an external URI or
    /// stable workspace id instead of the local directory name.
    pub subject: Option<String>,

    /// Decode files supported by `stencila-codecs` and add document subgraphs.
    ///
    /// Disabling decode keeps graph construction limited to the filesystem when
    /// callers want fast inventory data without reading document contents.
    pub decode: bool,

    /// Decode options passed to `stencila-codecs`.
    ///
    /// Forwarding codec options lets workspace graphs use the same decoding
    /// controls as direct codec callers.
    pub decode_options: Option<DecodeOptions>,

    /// Fail graph construction when a supported file cannot be decoded.
    ///
    /// The default is permissive so one bad document does not prevent a
    /// workspace inventory graph, but stricter callers can opt into failure.
    pub fail_on_decode_error: bool,

    /// Analyze supported environment manifests and lockfiles.
    ///
    /// Environment analysis is static: it reads manifests such as
    /// `pyproject.toml`, `package.json`, `Cargo.toml`, and `DESCRIPTION` but
    /// does not execute package managers or expand lockfiles.
    pub analyze_environment: bool,

    /// Fail graph construction when a supported environment manifest cannot be parsed.
    ///
    /// The default is permissive so one invalid manifest does not prevent a
    /// workspace inventory graph, but stricter callers can opt into failure.
    pub fail_on_environment_error: bool,
}

impl Default for WorkspaceOptions {
    fn default() -> Self {
        Self {
            subject: None,
            decode: true,
            decode_options: None,
            fail_on_decode_error: false,
            analyze_environment: true,
            fail_on_environment_error: false,
        }
    }
}

/// Build a graph from a workspace directory.
///
/// The workspace graph records directories, files, and decoded document graphs
/// so consumers can trace documents back to filesystem inputs.
///
/// Walking follows git ignore files, keeps hidden files, and skips common cache
/// or build directories such as `.git`, `.stencila`, `node_modules`, and
/// `target`.
pub async fn graph_from_path(
    root: impl AsRef<Path>,
    options: Option<WorkspaceOptions>,
) -> Result<Graph> {
    let root = root
        .as_ref()
        .canonicalize()
        .wrap_err_with(|| format!("unable to canonicalize {}", root.as_ref().display()))?;
    ensure!(
        root.is_dir(),
        "workspace graph root must be a directory: {}",
        root.display()
    );

    let options = options.unwrap_or_default();
    let subject = options.subject.clone().unwrap_or_else(|| {
        format!(
            "workspace:{}",
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(".")
        )
    });

    let mut builder = GraphBuilder::new(subject);
    let root_rel = WorkspaceRelPath::root();
    add_directory(&mut builder, &root_rel, &root);

    let entries = workspace_entries(&root)?;
    let entry_kinds = entries
        .iter()
        .map(|entry| (entry.rel.clone(), entry.kind))
        .collect::<BTreeMap<_, _>>();
    let source_files = entries
        .iter()
        .filter_map(|entry| {
            if entry.kind != WorkspaceEntryKind::File {
                return None;
            }

            let language = CodeLanguage::from_path(&entry.path)?;
            let code = std::fs::read_to_string(&entry.path).ok()?;
            Some((entry.rel.clone(), (language, code)))
        })
        .collect::<BTreeMap<_, _>>();
    let source_file_rels = source_files.keys().cloned().collect::<BTreeSet<_>>();

    for entry in entries {
        match entry.kind {
            WorkspaceEntryKind::Directory => {
                add_directory(&mut builder, &entry.rel, &entry.path);
                if let Some(parent) = entry.rel.parent() {
                    builder.add_containment(
                        LocalGraphId::directory(&entry.rel),
                        LocalGraphId::directory(&parent),
                        vec![evidence::observed()],
                    );
                }
            }
            WorkspaceEntryKind::File => {
                if options.analyze_environment {
                    let file_id_for_rel = |rel: &WorkspaceRelPath| {
                        matches!(entry_kinds.get(rel), Some(WorkspaceEntryKind::File))
                            .then(|| LocalGraphId::file(rel))
                    };
                    environment::add_environment_from_file(
                        &mut builder,
                        &entry.path,
                        &entry.rel,
                        file_id_for_rel,
                        options.fail_on_environment_error,
                    )?;
                }

                let source_id = if let Some((language, code)) = source_files.get(&entry.rel) {
                    let code_id = LocalGraphId::code_unit(entry.rel.as_str());
                    let parent_id = entry
                        .rel
                        .parent()
                        .map(|parent| LocalGraphId::directory(&parent));
                    let resolver = |literal: &str| {
                        workspace_reference_id(
                            &root_rel,
                            literal,
                            &entry_kinds,
                            &source_file_rels,
                            WorkspaceReferenceTarget::FileOrSymbolicLink,
                        )
                        .or_else(|| {
                            workspace_reference_id(
                                &entry.rel,
                                literal,
                                &entry_kinds,
                                &source_file_rels,
                                WorkspaceReferenceTarget::FileOrSymbolicLink,
                            )
                        })
                    };
                    code::add_workspace_code(
                        &mut builder,
                        code::WorkspaceCode {
                            unit_id: &code_id,
                            rel: &entry.rel,
                            code,
                            language: *language,
                            parent_id,
                            date_created: file_created_time(&entry.metadata),
                            date_modified: file_modified_time(&entry.metadata),
                        },
                        resolver,
                    );
                    code_id
                } else {
                    let file_id = add_file(&mut builder, &entry.path, &entry.rel, &entry.metadata);
                    if let Some(parent) = entry.rel.parent() {
                        builder.add_containment(
                            &file_id,
                            LocalGraphId::directory(&parent),
                            vec![evidence::observed()],
                        );
                    }
                    file_id
                };

                if options.decode
                    && decode_is_supported(&entry.path, options.decode_options.as_ref())
                {
                    match stencila_codecs::from_path_with_info(
                        &entry.path,
                        options.decode_options.clone(),
                    )
                    .await
                    {
                        Ok((node, ..)) => {
                            let mut reference_resolver = |kind, reference: &str| {
                                let file_id = workspace_reference_id(
                                    &entry.rel,
                                    reference,
                                    &entry_kinds,
                                    &source_file_rels,
                                    WorkspaceReferenceTarget::File,
                                )?;
                                let edge_kind = match kind {
                                    DocumentReferenceKind::Media => GraphEdgeKind::LinkedBy,
                                    DocumentReferenceKind::Include => GraphEdgeKind::IncludedBy,
                                    DocumentReferenceKind::Link => GraphEdgeKind::LinkedBy,
                                };

                                Some((file_id, edge_kind))
                            };
                            add_document_with_reference_resolver(
                                &mut builder,
                                entry.rel.as_str().to_string(),
                                &node,
                                Some(&source_id),
                                Some(&mut reference_resolver),
                            );
                        }
                        Err(error) if options.fail_on_decode_error => {
                            return Err(error).wrap_err_with(|| {
                                format!("unable to decode {}", entry.path.display())
                            });
                        }
                        Err(..) => {}
                    }
                }
            }
            WorkspaceEntryKind::SymbolicLink => {
                let symlink_id = add_symbolic_link(&mut builder, &entry.path, &entry.rel)?;
                if let Some(parent) = entry.rel.parent() {
                    builder.add_containment(
                        &symlink_id,
                        LocalGraphId::directory(&parent),
                        vec![evidence::observed()],
                    );
                }

                if let Some(target_id) =
                    symbolic_link_target_id(&root, &entry.path, &entry_kinds, &source_file_rels)?
                {
                    builder.add_link(target_id, symlink_id, evidence::observed_and_resolved());
                }
            }
            WorkspaceEntryKind::Other => {}
        }
    }

    builder.build()
}

/// Resolve a local reference to an existing workspace graph id.
fn workspace_reference_id(
    document_rel: &WorkspaceRelPath,
    reference: &str,
    entry_kinds: &BTreeMap<WorkspaceRelPath, WorkspaceEntryKind>,
    source_file_rels: &BTreeSet<WorkspaceRelPath>,
    target: WorkspaceReferenceTarget,
) -> Option<String> {
    let reference = reference.trim();
    if !is_local_relative_reference(reference) {
        return None;
    }

    for candidate in reference_path_candidates(reference) {
        let Some(rel) = document_relative_workspace_path(document_rel, &candidate) else {
            continue;
        };
        let Some(kind) = entry_kinds.get(&rel).copied() else {
            continue;
        };
        let id = match (target, kind) {
            (_, WorkspaceEntryKind::File) if source_file_rels.contains(&rel) => {
                Some(LocalGraphId::code_unit(rel.as_str()))
            }
            (_, WorkspaceEntryKind::File) => Some(LocalGraphId::file(&rel)),
            (WorkspaceReferenceTarget::FileOrSymbolicLink, WorkspaceEntryKind::SymbolicLink) => {
                Some(LocalGraphId::symbolic_link(&rel))
            }
            _ => None,
        };
        if id.is_some() {
            return id;
        }
    }

    None
}

/// Workspace node kinds that a local reference may resolve to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceReferenceTarget {
    /// Only concrete workspace files.
    File,

    /// Concrete files and symbolic link entries.
    FileOrSymbolicLink,
}

/// A walked filesystem entry that will be considered for graph output.
///
/// The entry records symlink-aware metadata and kind once, before graph
/// construction branches. This prevents accidental dereferencing through
/// `Path::is_file`, `Path::is_dir`, or `metadata` when a symbolic link is present.
struct WorkspaceEntry {
    /// Absolute path spelling returned by the workspace walker.
    path: PathBuf,

    /// Normalized path relative to the workspace root.
    rel: WorkspaceRelPath,

    /// Metadata for the directory entry itself.
    metadata: Metadata,

    /// The entry kind derived without dereferencing symbolic links.
    kind: WorkspaceEntryKind,
}

/// Filesystem entry kind used by workspace graph construction.
///
/// This is deliberately smaller than `std::fs::FileType`: the graph currently
/// represents directories, files, and symbolic links, while sockets, devices,
/// and other special entries are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceEntryKind {
    Directory,
    File,
    SymbolicLink,
    Other,
}

/// Collect workspace entries with symlink-aware metadata.
///
/// Building the entry list up front gives later symlink resolution a set of
/// graph-included target paths. That keeps links to skipped directories, missing
/// targets, or outside-workspace targets from producing dangling graph edges.
fn workspace_entries(root: &Path) -> Result<Vec<WorkspaceEntry>> {
    workspace_paths(root)?
        .into_iter()
        .map(|path| {
            let rel = WorkspaceRelPath::from_workspace_path(root, &path)?;
            let metadata = path
                .symlink_metadata()
                .wrap_err_with(|| format!("unable to read metadata for {}", path.display()))?;
            let kind = WorkspaceEntryKind::from_metadata(&metadata);

            Ok(WorkspaceEntry {
                path,
                rel,
                metadata,
                kind,
            })
        })
        .collect()
}

impl WorkspaceEntryKind {
    /// Classify metadata without following symbolic links.
    ///
    /// Symbolic links must be checked first because path helpers such as
    /// `Path::is_file` follow links and would make a link to a file look like a
    /// real workspace file.
    fn from_metadata(metadata: &Metadata) -> Self {
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            Self::SymbolicLink
        } else if file_type.is_dir() {
            Self::Directory
        } else if file_type.is_file() {
            Self::File
        } else {
            Self::Other
        }
    }
}

/// Determine whether a path should be decoded with the supplied options.
///
/// This mirrors `stencila-codecs` path decoding: explicit codec and format
/// options override path-derived inference, while files without explicit decode
/// options are skipped when their extension is unsupported.
fn decode_is_supported(path: &Path, decode_options: Option<&DecodeOptions>) -> bool {
    let codec = decode_options.and_then(|options| options.codec.as_ref());
    let format = decode_options
        .and_then(|options| options.format.clone())
        .unwrap_or_else(|| Format::from_path(path));

    stencila_codecs::get(codec, Some(&format), Some(CodecDirection::Decode)).is_ok()
}

/// Collect paths that belong to a workspace.
///
/// This helper keeps ignore handling and deterministic sorting in one place so
/// graph output is stable and follows the same skip rules for every caller.
fn workspace_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| {
            entry.depth() == 0
                || !matches!(entry.file_name().to_str(), Some(name) if is_skipped_name(name))
        });

    let mut paths = Vec::new();
    for entry in builder.build() {
        let entry =
            entry.wrap_err_with(|| format!("unable to walk workspace {}", root.display()))?;
        if entry.depth() > 0 {
            paths.push(entry.into_path());
        }
    }
    paths.sort();

    Ok(paths)
}

/// Check whether a path name should be skipped.
///
/// These names are build products, caches, or internal state that would add
/// noisy graph nodes without representing authored workspace content.
fn is_skipped_name(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".stencila"
            | ".ruff_cache"
            | ".pytest_cache"
            | ".mypy_cache"
            | "__pycache__"
            | "node_modules"
            | "target"
    )
}

/// Add a directory node to the graph.
///
/// Directory nodes make filesystem containment explicit, allowing files,
/// symbolic links, and subdirectories to be related with `PartOf` edges instead
/// of nested payloads.
fn add_directory(builder: &mut GraphBuilder, rel: &WorkspaceRelPath, path: &Path) {
    let id = LocalGraphId::directory(rel);
    let name = if rel.as_str() == "." {
        path_name(path, ".")
    } else {
        path_name(path, "")
    };

    let mut directory = Directory::new(name, rel.as_str().to_string());
    directory.id = Some(id.clone());
    builder.add_schema_node(id, Node::Directory(directory));
}

/// Add a file node to the graph.
///
/// File nodes capture stable workspace-relative identity and lightweight
/// metadata so decoded document graphs can point back to their source files.
fn add_file(
    builder: &mut GraphBuilder,
    path: &Path,
    rel: &WorkspaceRelPath,
    metadata: &Metadata,
) -> String {
    let id = LocalGraphId::file(rel);
    let name = path_name(path, "");
    let size = metadata.len();

    let mut file = File::new(name, rel.as_str().to_string());
    file.id = Some(id.clone());
    file.size = Some(size);
    file.options.date_created = file_created_time(metadata);
    file.options.date_modified = file_modified_time(metadata);
    if let Some(identifier) = environment::file_digest_identifier(path, rel) {
        file.options.identifiers = Some(vec![identifier]);
    }

    builder.add_schema_node(id.clone(), Node::File(file));
    id
}

/// Add a symbolic link node to the graph.
///
/// Symbolic links are represented as the link entry itself. The target path is
/// stored exactly as the filesystem reports it, but the graph does not decode
/// through the link or treat the link as the target file/directory.
fn add_symbolic_link(
    builder: &mut GraphBuilder,
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<String> {
    let id = LocalGraphId::symbolic_link(rel);
    let name = path_name(path, "");
    let target = path
        .read_link()
        .wrap_err_with(|| format!("unable to read symbolic link {}", path.display()))?;
    let target_path = filesystem_path_string(&target)?;

    let mut symlink = SymbolicLink::new(name, rel.as_str().to_string(), target_path);
    symlink.id = Some(id.clone());

    builder.add_schema_node(id.clone(), Node::SymbolicLink(symlink));
    Ok(id)
}

/// Find the graph id for a symbolic link target when it is in this graph.
///
/// This uses lexical path resolution rather than canonicalization so a link to
/// another symlink in the workspace points at that symlink entry, not at the
/// final dereferenced destination. Targets outside the workspace, missing
/// targets, and targets under skipped directories intentionally produce no edge.
fn symbolic_link_target_id(
    root: &Path,
    link_path: &Path,
    entry_kinds: &BTreeMap<WorkspaceRelPath, WorkspaceEntryKind>,
    source_file_rels: &BTreeSet<WorkspaceRelPath>,
) -> Result<Option<String>> {
    let target = link_path
        .read_link()
        .wrap_err_with(|| format!("unable to read symbolic link {}", link_path.display()))?;
    let target_path = if target.is_absolute() {
        target
    } else {
        link_path.parent().unwrap_or(root).join(target)
    };
    let target_path = normalize_path_lexically(&target_path);
    let Ok(rel) = WorkspaceRelPath::from_workspace_path(root, &target_path) else {
        return Ok(None);
    };

    let Some(kind) = entry_kinds.get(&rel).copied().or_else(|| {
        if rel.as_str() == "." {
            Some(WorkspaceEntryKind::Directory)
        } else {
            None
        }
    }) else {
        return Ok(None);
    };

    Ok(match kind {
        WorkspaceEntryKind::Directory => Some(LocalGraphId::directory(&rel)),
        WorkspaceEntryKind::File if source_file_rels.contains(&rel) => {
            Some(LocalGraphId::code_unit(rel.as_str()))
        }
        WorkspaceEntryKind::File => Some(LocalGraphId::file(&rel)),
        WorkspaceEntryKind::SymbolicLink => Some(LocalGraphId::symbolic_link(&rel)),
        WorkspaceEntryKind::Other => None,
    })
}

/// Convert a filesystem path into a schema string without lossy replacement.
///
/// Schema path fields are strings, so non-UTF-8 symlink targets are rejected
/// rather than silently converting bytes to replacement characters and making
/// graph ids or target metadata ambiguous.
fn filesystem_path_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| eyre::eyre!("workspace graph paths must be UTF-8: {}", path.display()))
}

/// Read a file creation time when the filesystem exposes it.
fn file_created_time(metadata: &Metadata) -> Option<SchemaDateTime> {
    metadata.created().ok().map(Into::into)
}

/// Read a file modification time.
fn file_modified_time(metadata: &Metadata) -> Option<SchemaDateTime> {
    metadata.modified().ok().map(Into::into)
}

/// Read a path's file name as UTF-8 with a fallback.
fn path_name(path: &Path, default: &str) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(default)
        .to_string()
}
