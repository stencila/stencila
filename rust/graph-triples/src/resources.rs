use derivative::Derivative;
use eyre::Result;
use path_slash::PathExt;
use schemars::JsonSchema;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Resource {
    /// A symbol within code, within a project file
    Symbol(Symbol),

    /// A node within a project file
    Node(Node),

    /// A file within the project
    File(File),

    /// A declared project `Source`
    Source(Source),

    /// A programming language module, usually part of an external package
    Module(Module),

    /// A URL to a remote resource
    Url(Url),
}

/// The id of a resource
pub type ResourceId = String;

impl Resource {
    /// Get the resource id
    pub fn id(&self) -> String {
        match self {
            Resource::Symbol(Symbol { path, name, .. }) => {
                ["symbol:", &path.display().to_string(), "@", name].concat()
            }
            Resource::Node(Node { path, id, .. }) => {
                ["node:", &path.display().to_string(), "#", id].concat()
            }
            Resource::File(File { path, .. }) => ["file:", &path.display().to_string()].concat(),
            Resource::Source(Source { name, .. }) => ["source:", name].concat(),
            Resource::Module(Module { language, name, .. }) => {
                ["module:", language, "::", name].concat()
            }
            Resource::Url(Url { url }) => url.clone(),
        }
    }
}


/// An entry for a resource in a topological sort of a dependency graph
#[derive(Debug, Clone, Serialize)]
pub struct ResourceEntry {
    /// The id of the resource
    pub id: String,

    /// The resource
    pub resource: Resource,

    /// The ids of any dependencies in the dependency graph
    pub dependencies: Vec<String>,

    /// The depth of the resource in the dependency graph.
    /// 
    /// A resource that has no dependencies has a depth of zero.
    /// Otherwise the depth is the maximum depth of dependencies plus one.
    pub depth: usize
}


#[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
#[derivative(PartialEq, Eq, Hash)]
#[schemars(deny_unknown_fields)]
pub struct Symbol {
    /// The path of the file that the symbol is defined in
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,

    /// The name/identifier of the symbol
    pub name: String,

    /// The type of the object that the symbol refers to (e.g `Number`, `Function`)
    ///
    /// Should be used as a hint only, and as such is excluded from
    /// equality and hash functions.
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub kind: String,
}

/// Create a new `Symbol` resource
pub fn symbol(path: &Path, name: &str, kind: &str) -> Resource {
    Resource::Symbol(Symbol {
        path: path.to_path_buf(),
        name: name.into(),
        kind: kind.into(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Node {
    /// The path of the file that the node is defined in
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,

    /// The id of the node with the document
    pub id: String,

    /// The type of node e.g. `Parameter`, `CodeChunk`
    pub kind: String,
}

/// Create a new `Node` resource
pub fn node(path: &Path, id: &str, kind: &str) -> Resource {
    Resource::Node(Node {
        path: path.to_path_buf(),
        id: id.into(),
        kind: kind.into(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct File {
    /// The path of the file
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
}

/// Create a new `File` resource
pub fn file(path: &Path) -> Resource {
    Resource::File(File {
        path: path.to_path_buf(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Source {
    /// The name of the project source
    pub name: String,
}

/// Create a new `Source` resource
pub fn source(name: &str) -> Resource {
    Resource::Source(Source { name: name.into() })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Module {
    /// The programming language of the module
    pub language: String,

    /// The name of the module
    pub name: String,
}

/// Create a new `Module` resource
pub fn module(language: &str, name: &str) -> Resource {
    Resource::Module(Module {
        language: language.into(),
        name: name.into(),
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Url {
    /// The URL of the external resource
    pub url: String,
}

/// Create a new `Url` resource
pub fn url(url: &str) -> Resource {
    Resource::Url(Url { url: url.into() })
}

/// Serialize the `path` fields of resources so that they use Unix forward slash
/// separators on all platforms.
fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    path.to_slash_lossy().serialize(serializer)
}
