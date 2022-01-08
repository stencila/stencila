use derivative::Derivative;
use eyre::Result;
use hash_utils::{file_sha256_hex, str_sha256_hex};
use path_slash::PathExt;
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{Pairs, Relation};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Resource {
    /// A symbol within code, within a document
    Symbol(Symbol),

    /// A node containing code, or associated with code, within a document
    Code(Code),

    /// A node within a document
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
    /// Get the [`ResourceId`] for a resource
    pub fn resource_id(&self) -> ResourceId {
        match self {
            Resource::Symbol(Symbol { path, name, .. }) => {
                ["symbol://", &path.to_slash_lossy(), "#", name].concat()
            }
            Resource::Code(Code { path, id, .. }) => {
                ["code://", &path.to_slash_lossy(), "#", id].concat()
            }
            Resource::Node(Node { path, id, .. }) => {
                ["node://", &path.to_slash_lossy(), "#", id].concat()
            }
            Resource::File(File { path, .. }) => ["file://", &path.to_slash_lossy()].concat(),
            Resource::Source(Source { name, .. }) => ["source://", name].concat(),
            Resource::Module(Module { language, name, .. }) => {
                ["module://", language, "#", name].concat()
            }
            Resource::Url(Url { url }) => url.clone(),
        }
    }

    /// Generate a `compile_digest` for a resource
    pub fn compile_digest(&self) -> String {
        match self {
            Resource::File(File { path }) => {
                file_sha256_hex(path).unwrap_or_else(|_| str_sha256_hex(&self.resource_id()))
            }
            _ => str_sha256_hex(&self.resource_id()),
        }
    }

    /// Get the [`ResourceInfo`] for a resource
    pub fn resource_info(&self) -> ResourceInfo {
        ResourceInfo::new(self.clone(), None, None, Some(self.compile_digest()))
    }

    /// Get the [`NodeId`] for resources that have it
    pub fn node_id(&self) -> Option<String> {
        match self {
            Resource::Code(Code { id, .. }) | Resource::Node(Node { id, .. }) => Some(id.clone()),
            _ => None,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct ResourceInfo {
    /// The resource (the "subject") that this information is for
    pub resource: Resource,

    /// The [`Relation`]-[`Resource`] pairs between the resource (the "subject") and
    /// other resources (the "objects").
    ///
    /// This is the primary data used to build the dependency graph between resources.
    pub relations: Option<Pairs>,

    /// The dependencies of the resource
    pub dependencies: Option<Vec<Resource>>,

    /// The depth of the resource in the dependency graph.
    ///
    /// A resource that has no dependencies has a depth of zero.
    /// Otherwise the depth is the maximum depth of dependencies plus one.
    pub depth: Option<usize>,

    /// Whether the resource is explicitly marked as pure or impure
    ///
    /// Pure resources do not modify other resources (i.e. they have no side effects).
    /// This can be determined from whether the resource has any `Assign`, `Alter` or `Write`
    /// relations. Additionally, the user may mark the resource as pure or impure
    /// for example, by using `@pure` or `@impure` tags in code comments.
    pub pure: Option<bool>,

    /// A digest of the resource when it was compiled
    ///
    /// This digest is intended to capture the "semantic intent" of the resource
    /// with respect to the dependency graph. For example, for `Code` resources
    /// it is preferably derived from the AST of the code and should only change
    /// when the semantics of the code change. For `File` resources, this may be
    /// a hash digest of the entire file, or of it's modification time for large files.
    pub compile_digest: Option<String>,

    /// A digest of the resource when it was linked with other resources
    pub link_digest: Option<String>,

    /// A digest of the resource the last time that it was executed
    pub execute_digest: Option<String>,
}

impl ResourceInfo {
    /// Create a new `ResourceInfo` object
    pub fn new(
        resource: Resource,
        relations: Option<Pairs>,
        pure: Option<bool>,
        compile_digest: Option<String>,
    ) -> Self {
        Self {
            resource,
            relations,
            dependencies: None,
            depth: None,
            pure,
            compile_digest,
            link_digest: None,
            execute_digest: None,
        }
    }

    /// Create a SHA256 hash digest from a value
    ///
    /// Suitable for use when generating the `_digest` properties of a [`ResourceInfo`]
    /// object.
    pub fn sha256_digest<T: Display>(value: &T) -> String {
        str_sha256_hex(&value.to_string())
    }

    /// Is the resource pure (i.e. has no side effects)?
    ///
    /// If the resource has not been explicitly tagged as pure or impure then
    /// returns `true` if there are any side-effect causing relations.
    pub fn is_pure(&self) -> bool {
        self.pure.unwrap_or_else(|| match &self.relations {
            Some(relations) => {
                relations
                    .iter()
                    .filter(|(relation, ..)| {
                        matches!(
                            relation,
                            Relation::Assign(..)
                                | Relation::Alter(..)
                                | Relation::Import(..)
                                | Relation::Write(..)
                        )
                    })
                    .count()
                    == 0
            }
            None => false,
        })
    }

    /// Get a list of symbols used by the resource
    pub fn symbols_used(&self) -> Vec<Symbol> {
        match &self.relations {
            Some(relations) => relations
                .iter()
                .filter_map(|pair| match pair {
                    (Relation::Use(..), Resource::Symbol(symbol)) => Some(symbol),
                    _ => None,
                })
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get a list of symbols modified by the resource
    pub fn symbols_modified(&self) -> Vec<Symbol> {
        match &self.relations {
            Some(relations) => relations
                .iter()
                .filter_map(|pair| match pair {
                    (Relation::Assign(..), Resource::Symbol(symbol))
                    | (Relation::Alter(..), Resource::Symbol(symbol)) => Some(symbol),
                    _ => None,
                })
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[derivative(PartialOrd = "ignore")]
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

#[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[schemars(deny_unknown_fields)]
pub struct Node {
    /// The path of the file that the node is defined in
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,

    /// The id of the node with the document
    pub id: String,

    /// The type of node e.g. `Link`, `ImageObject`
    ///
    /// Should be used as a hint only, and as such is excluded from
    /// equality and hash functions.
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
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

#[skip_serializing_none]
#[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[schemars(deny_unknown_fields)]
pub struct Code {
    /// The path of the file that the node is defined in
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,

    /// The id of the node with the document
    pub id: String,

    /// The type of node e.g. `Parameter`, `CodeChunk`
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    pub kind: String,

    /// The programming language associated with the node (if any)
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    pub language: Option<String>,
}

/// Create a new `Code` resource
pub fn code(path: &Path, id: &str, kind: &str, language: Option<String>) -> Resource {
    Resource::Code(Code {
        path: path.to_path_buf(),
        id: id.into(),
        kind: kind.into(),
        language,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Source {
    /// The name of the project source
    pub name: String,
}

/// Create a new `Source` resource
pub fn source(name: &str) -> Resource {
    Resource::Source(Source { name: name.into() })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
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
