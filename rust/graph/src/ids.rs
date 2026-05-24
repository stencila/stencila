//! Graph-local identifiers and workspace path normalization.
//!
//! Graph endpoints are intentionally local to a `Graph`: they are stable labels
//! for edges, not URLs that should be dereferenced directly. Keeping the grammar
//! here avoids every graph collector inventing its own string format and makes
//! the path boundary explicit before paths are embedded in ids.

use std::{
    fmt::Write,
    path::{Component, Path},
};

use eyre::{Result, bail};
use stencila_schema::NodeId;

/// A normalized path relative to the workspace root.
///
/// Workspace graphs need path ids to be portable and deterministic, but `Path`
/// itself is platform-shaped and can represent absolute paths, parent
/// traversals, and non-UTF-8 names. This wrapper records the narrower contract
/// used by graph ids and embedded `File.path`, `Directory.path`, and
/// `SymbolicLink.path` fields: workspace-relative, UTF-8 path components joined
/// with `/`, with `.` reserved for the workspace root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct WorkspaceRelPath(String);

impl WorkspaceRelPath {
    /// Construct the relative path for the workspace root.
    ///
    /// The root is represented as `.` because an empty string is easy to lose in
    /// serialized output and makes parent/containment relationships harder to
    /// inspect by eye.
    pub(crate) fn root() -> Self {
        Self(".".to_string())
    }

    /// Convert an absolute workspace entry path into a normalized relative path.
    ///
    /// This is the main boundary check for workspace graph construction: callers
    /// pass a canonical workspace root and a walked entry, and this function
    /// refuses any spelling that does not sit under that root before ids are
    /// created from it.
    pub(crate) fn from_workspace_path(root: &Path, path: &Path) -> Result<Self> {
        let relative = path.strip_prefix(root).map_err(|error| {
            eyre::eyre!(
                "{} is not within {}: {error}",
                path.display(),
                root.display()
            )
        })?;

        Self::from_relative_path(relative)
    }

    /// Normalize a path that is already known to be relative.
    ///
    /// The graph uses `/` as the separator even on Windows so snapshots and
    /// downstream cache keys do not vary by operating system. Each component is
    /// preserved as text; delimiter escaping is applied later only when the path
    /// is placed inside a graph id.
    pub(crate) fn from_relative_path(path: &Path) -> Result<Self> {
        if path.as_os_str().is_empty() {
            return Ok(Self::root());
        }

        let mut parts = Vec::new();
        for component in path.components() {
            match component {
                Component::Normal(part) => {
                    let part = part.to_str().ok_or_else(|| {
                        eyre::eyre!("workspace graph paths must be UTF-8: {}", path.display())
                    })?;
                    parts.push(part.to_string());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    bail!(
                        "workspace graph paths must not contain `..`: {}",
                        path.display()
                    );
                }
                Component::RootDir | Component::Prefix(..) => {
                    bail!(
                        "workspace graph paths must be relative to the workspace root: {}",
                        path.display()
                    );
                }
            }
        }

        if parts.is_empty() {
            Ok(Self::root())
        } else {
            Ok(Self(parts.join("/")))
        }
    }

    /// Return the normalized relative path string.
    ///
    /// This is suitable for schema path fields. It is intentionally not escaped,
    /// because those fields are paths, not graph-id components.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the normalized parent directory path.
    ///
    /// Files and directories are emitted as flat graph nodes, so containment
    /// edges need a consistent way to find the parent node id while still keeping
    /// the workspace root as a real node.
    pub(crate) fn parent(&self) -> Option<Self> {
        if self.0 == "." {
            return None;
        }

        match self.0.rsplit_once('/') {
            Some((parent, _name)) if !parent.is_empty() => Some(Self(parent.to_string())),
            _ => Some(Self::root()),
        }
    }

    /// Return the path escaped for use inside a graph-local id.
    fn encoded_for_id(&self) -> String {
        encode_id_component(&self.0)
    }
}

/// Constructors for graph-local ids.
///
/// These ids are optimized for readability in serialized graphs while remaining
/// unambiguous enough for tools to parse if they need to. The grammar is:
///
/// Workspace filesystem entries:
///
/// - `dir:<path>`
/// - `file:<path>`
/// - `symlink:<path>`
/// - `code:<scope>`
/// - `datatable:<path>`
/// - `image:<path>`
/// - `audio:<path>`
/// - `video:<path>`
///
/// Software environments:
///
/// - `environment:<ecosystem>:<path>`
/// - `package:<ecosystem>/<name>`
///
/// Document graph nodes and references:
///
/// - `node:<scope>#<node-id>`
/// - `node:<scope>` for a root node fallback when the schema node has no id
/// - `action:execute:<scope>#<node-id>`
/// - `output:<scope>#<node-id>:<index>`
/// - `reference:<scope>#<citation-target>`
/// - `resource:<uri>`
///
/// Static code analysis nodes:
///
/// - `file-ref:<scope>:<path>`
/// - `symbol:<scope>:<language>:<name>`
/// - `function:<scope>:<language>:<name>`
/// - `workflow-unit:<scope>:<name>`
/// - `column:<scope>:<dataframe>:<name>`
///
/// Dynamic components are percent-encoded, keeping `/` visible in workspace
/// paths but escaping delimiters such as `:`, `#`, `%`, whitespace, controls,
/// and non-ASCII bytes.
pub(crate) struct LocalGraphId;

impl LocalGraphId {
    /// Create the graph id for a directory node.
    ///
    /// Prefixing by node kind lets files and directories share the same path
    /// spelling without colliding in edge endpoints.
    pub(crate) fn directory(path: &WorkspaceRelPath) -> String {
        format!("dir:{}", path.encoded_for_id())
    }

    /// Create the graph id for a file node.
    ///
    /// File ids use the workspace-relative path because paths are the durable
    /// identity available before a document is decoded and stabilized.
    pub(crate) fn file(path: &WorkspaceRelPath) -> String {
        format!("file:{}", path.encoded_for_id())
    }

    /// Create the graph id for a symbolic link node.
    ///
    /// Symbolic links get their own prefix because the link path is a filesystem
    /// entry in its own right and should not be confused with the file or
    /// directory that its target path may resolve to.
    pub(crate) fn symbolic_link(path: &WorkspaceRelPath) -> String {
        format!("symlink:{}", path.encoded_for_id())
    }

    /// Create the graph id for a source-code file represented as a SoftwareSourceCode node.
    pub(crate) fn code(scope: &str) -> String {
        format!("code:{}", encode_id_component(scope))
    }

    /// Create the graph id for a tabular data file represented as a Datatable.
    pub(crate) fn datatable(path: &WorkspaceRelPath) -> String {
        format!("datatable:{}", path.encoded_for_id())
    }

    /// Create the graph id for an image file represented as an ImageObject.
    pub(crate) fn image(path: &WorkspaceRelPath) -> String {
        format!("image:{}", path.encoded_for_id())
    }

    /// Create the graph id for an audio file represented as an AudioObject.
    pub(crate) fn audio(path: &WorkspaceRelPath) -> String {
        format!("audio:{}", path.encoded_for_id())
    }

    /// Create the graph id for a video file represented as a VideoObject.
    pub(crate) fn video(path: &WorkspaceRelPath) -> String {
        format!("video:{}", path.encoded_for_id())
    }

    /// Create the graph id for an environment declared by a manifest file.
    pub(crate) fn environment(ecosystem: &str, path: &WorkspaceRelPath) -> String {
        format!(
            "environment:{}:{}",
            encode_id_component(ecosystem),
            path.encoded_for_id()
        )
    }

    /// Create the graph id for an imported or declared software package.
    pub(crate) fn package(name: &str) -> String {
        format!("package:{}", encode_id_component(name))
    }

    /// Create the graph id for a schema node inside a scoped document graph.
    ///
    /// The `#` separator mirrors familiar fragment syntax but remains local to
    /// this graph id grammar; both sides are encoded before being joined.
    pub(crate) fn document_node(scope: &str, node_id: &NodeId) -> String {
        format!(
            "node:{}#{}",
            encode_id_component(scope),
            encode_id_component(&node_id.to_string())
        )
    }

    /// Create the fallback id for a document root without a schema node id.
    ///
    /// Most document roots are stabilized with ids before graph collection, but
    /// this fallback keeps the public document helper total for primitive or
    /// otherwise id-less nodes.
    pub(crate) fn document_root(scope: &str) -> String {
        format!("node:{}", encode_id_component(scope))
    }

    /// Create a stable id for an execution action attached to an edge.
    ///
    /// Execution actions are edge metadata, not graph endpoints, but a stable id
    /// keeps repeated serializations easy to compare and reference.
    pub(crate) fn execute_action(scope: &str, node_id: &NodeId) -> String {
        format!(
            "action:execute:{}#{}",
            encode_id_component(scope),
            encode_id_component(&node_id.to_string())
        )
    }

    /// Create the fallback id for an execution output without its own node id.
    ///
    /// Outputs are indexed under the executable node id so repeated executions of
    /// the same stabilized document produce the same graph shape.
    pub(crate) fn output(scope: &str, node_id: &NodeId, index: usize) -> String {
        format!(
            "output:{}#{}:{index}",
            encode_id_component(scope),
            encode_id_component(&node_id.to_string())
        )
    }

    /// Create the graph id for a cited reference keyed by citation target.
    ///
    /// Citation targets are often bibliography keys, DOI strings, or URLs rather
    /// than Stencila node ids. Keeping them in their own namespace avoids
    /// conflating bibliographic resources with document nodes.
    pub(crate) fn reference(scope: &str, target: &str) -> String {
        format!(
            "reference:{}#{}",
            encode_id_component(scope),
            encode_id_component(target)
        )
    }

    /// Create the graph id for an external linked resource.
    ///
    /// The resource namespace is intentionally graph-local. It represents the
    /// target URI as a relationship endpoint without asserting that Stencila has
    /// fetched or decoded the resource.
    pub(crate) fn resource(uri: &str) -> String {
        format!("resource:{}", encode_id_component(uri))
    }

    /// Create the graph id for an unresolved file reference discovered in code.
    ///
    /// Workspace graph construction resolves references to concrete workspace
    /// resource ids when possible. This constructor is for the remaining
    /// synthetic `File` nodes scoped under the analyzed code unit.
    pub(crate) fn file_ref(scope: &str, path: &str) -> String {
        format!(
            "file-ref:{}:{}",
            encode_id_component(scope),
            encode_id_component(path)
        )
    }

    /// Create the graph id for a symbol in a source-code scope.
    pub(crate) fn symbol(scope: &str, language: &str, name: &str) -> String {
        format!(
            "symbol:{}:{}:{}",
            encode_id_component(scope),
            encode_id_component(language),
            encode_id_component(name)
        )
    }

    /// Create the graph id for a function-like callable in a source-code scope.
    pub(crate) fn function(scope: &str, language: &str, name: &str) -> String {
        format!(
            "function:{}:{}:{}",
            encode_id_component(scope),
            encode_id_component(language),
            encode_id_component(name)
        )
    }

    /// Create the graph id for a workflow unit discovered in code.
    ///
    /// The schema currently represents these as `Function` nodes, but they get
    /// their own namespace because workflow schedulers give them graph semantics
    /// ordinary functions do not have: they are named execution units with
    /// rule/process-level inputs, outputs, and script links. Keeping the
    /// namespace separate lets projections keep workflow structure visible
    /// without also showing every local function call.
    ///
    /// The neutral `workflow-unit` namespace covers both Snakemake rules and
    /// Nextflow processes.
    pub(crate) fn workflow_unit(scope: &str, name: &str) -> String {
        format!(
            "workflow-unit:{}:{}",
            encode_id_component(scope),
            encode_id_component(name)
        )
    }

    /// Create the graph id for a dataframe column discovered in code.
    pub(crate) fn column(scope: &str, origin: &str, name: &str) -> String {
        format!(
            "column:{}:{}:{}",
            encode_id_component(scope),
            encode_id_component(origin),
            encode_id_component(name)
        )
    }
}

/// Percent-encode one graph-id component.
///
/// The graph id grammar uses `:`, `#`, and `%` as delimiters/escape markers, so
/// those bytes must never appear raw inside dynamic components. Encoding all
/// bytes except URI-unreserved characters and `/` keeps common paths legible
/// while still making the grammar reversible for tooling that wants to parse it.
fn encode_id_component(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(*byte as char)
            }
            byte => {
                let _ = write!(&mut encoded, "%{byte:02X}");
            }
        }
    }

    encoded
}
