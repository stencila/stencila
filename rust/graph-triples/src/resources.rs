use derivative::Derivative;
use eyre::Result;
use hash_utils::{file_sha256, str_sha256};
use path_slash::PathExt;
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use stencila_schema::{CodeChunkExecuteAuto, CodeExecutableExecuteStatus};

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

    /// Generate a [`ResourceDigest`] for a resource.
    ///
    /// If the resource variant does not support generation of a digest,
    /// a default (empty) digest is returned.
    pub fn digest(&self) -> ResourceDigest {
        match self {
            Resource::File(File { path }) => ResourceDigest::from_file(path),
            _ => ResourceDigest::default(),
        }
    }

    /// Get the [`ResourceInfo`] for a resource
    pub fn resource_info(&self) -> ResourceInfo {
        ResourceInfo::new(
            self.clone(),
            None,
            None,
            None,
            Some(self.digest()),
            None,
            None,
        )
    }

    /// Get the type of [`Node`] for resources that have it
    pub fn node_type(&self) -> Option<&str> {
        match self {
            Resource::Code(Code { kind, .. }) | Resource::Node(Node { kind, .. }) => {
                Some(kind.as_str())
            }
            _ => None,
        }
    }

    /// Get the [`NodeId`] for resources that have it
    pub fn node_id(&self) -> Option<&str> {
        match self {
            Resource::Code(Code { id, .. }) | Resource::Node(Node { id, .. }) => Some(id.as_str()),
            _ => None,
        }
    }
}
/// A digest representing the state of a [`Resource`] and its dependencies.
///
/// The digest is separated into several parts. Although initially it may seem that the
/// parts are redundant ("can't they all be folded into a single digest?"), each
/// part provides useful information. For example, it is useful to store
/// the `content_digest`, in addition to `semantic_digest`, to be able
/// to indicate to the user that a change in the resource has been detected but
/// that it does not appear to change its semantics.
#[derive(Debug, Default, Clone)]
pub struct ResourceDigest {
    /// A digest that captures the content of the resource (e.g the `text`
    /// of a `CodeChunk`, or the bytes of a file).
    pub content_digest: String,

    /// A digest that captures the "semantic intent" of the resource
    /// with respect to the dependency graph.
    ///
    /// For example, for `Code` resources it is preferably derived from the AST
    /// of the code and should only change when the semantics of the code change.
    pub semantic_digest: String,

    /// A digest of the `dependencies_digest`s of the dependencies of a resource.
    ///
    /// If there are no dependencies then `dependencies_digest` is an empty string.
    pub dependencies_digest: String,

    /// The count of the number of code dependencies that are stale (i.e. are out of sync with the `KernelSpace`).
    ///
    /// If there are no dependencies then `dependencies_stale` is zero. May include
    /// duplicates for diamond shaped dependency graphs so this represents a maximum number.
    pub dependencies_stale: u32,

    /// The count of the number of code dependencies that had `execute_status == Failed`
    ///
    /// If there are no dependencies then `dependencies_failed` is zero. May include
    /// duplicates for diamond shaped dependency graphs so this represents a maximum number.
    pub dependencies_failed: u32,
}

impl ResourceDigest {
    /// Create a new `ResourceDigest` from its string representation
    pub fn from_string(string: &str) -> Self {
        let parts: Vec<&str> = string.split('.').collect();
        let content_digest = parts.get(0).map_or_else(String::new, |str| str.to_string());
        let semantic_digest = parts.get(1).map_or_else(String::new, |str| str.to_string());
        let dependencies_digest = parts.get(2).map_or_else(String::new, |str| str.to_string());
        let dependencies_stale = parts
            .get(3)
            .map_or(0, |str| str.parse().unwrap_or_default());
        let dependencies_failed = parts
            .get(4)
            .map_or(0, |str| str.parse().unwrap_or_default());
        Self {
            content_digest,
            semantic_digest,
            dependencies_digest,
            dependencies_stale,
            dependencies_failed,
        }
    }

    /// Create a new `ResourceDigest` from strings for content and semantics.
    ///
    /// Before generating the hash of strings remove carriage returns from strings to avoid
    /// cross platform differences in generated digests.
    pub fn from_strings(content_str: &str, semantic_str: Option<&str>) -> Self {
        let content_digest = Self::base64_encode(&str_sha256(&Self::strip_chars(content_str)));
        let semantic_digest = semantic_str.map_or_else(String::default, |str| {
            Self::base64_encode(&str_sha256(&Self::strip_chars(str)))
        });
        Self {
            content_digest,
            semantic_digest,
            ..Default::default()
        }
    }

    /// Create a new `ResourceDigest` from a file
    ///
    /// If there is an error when hashing the file, a default (empty) digest is returned.
    pub fn from_file(path: &Path) -> Self {
        match file_sha256(path) {
            Ok(bytes) => Self::from_bytes(&bytes, None),
            Err(..) => Self::default(),
        }
    }

    /// Create a new `ResourceDigest` from bytes for content and semantics
    ///
    /// To minimize the size of the digest while maintaining uniqueness, the bytes are usually,
    /// but not necessarily, the output of a hashing function.
    pub fn from_bytes(content_bytes: &[u8], semantic_bytes: Option<&[u8]>) -> Self {
        let content_digest = Self::base64_encode(content_bytes);
        let semantic_digest = semantic_bytes.map_or_else(String::default, Self::base64_encode);
        Self {
            content_digest,
            semantic_digest,
            ..Default::default()
        }
    }

    /// Strip carriage returns (and possibly other problematic characters) from strings
    pub fn strip_chars(bytes: &str) -> String {
        bytes.replace("\r", "")
    }

    /// Encode bytes as Base64
    ///
    /// Uses a URL safe (https://tools.ietf.org/html/rfc3548#section-4) character set
    /// and does not include padding (because it is unnecessary in this use case).
    pub fn base64_encode(bytes: &[u8]) -> String {
        base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
    }
}

// String representation of `ResourceDigest`
impl Display for ResourceDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}.{}.{}.{}.{}",
            self.content_digest,
            self.semantic_digest,
            self.dependencies_digest,
            self.dependencies_stale,
            self.dependencies_failed
        )
    }
}

// Use `Display` for serialization
impl Serialize for ResourceDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
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
    ///
    /// Derived during graph `update()`.
    /// Used when generating an execution plan to determine which of
    /// a resource's dependencies need to be executed as well.
    pub dependencies: Option<Vec<Resource>>,

    /// The direct dependents of the resource
    ///
    /// Derived during graph `update()`.
    /// However, since that is done in topological order, we are unable to get all dependents.
    /// Doing so would require `update()` to be more time consuming, so at this stage we're avoiding that.
    pub dependents: Option<Vec<Resource>>,

    /// The depth of the resource in the dependency graph.
    ///
    /// Derived during graph `update()` from the depths of the
    /// resource's `dependencies`.
    /// A resource that has no dependencies has a depth of zero.
    /// Otherwise, the depth is the maximum depth of dependencies plus one.
    pub depth: Option<usize>,

    /// Under which circumstances the resource should be automatically executed
    ///
    /// In the below descriptions:
    ///
    /// - "run" means that the user made an explicit request to execute the specific resource
    ///   (e.g. presses the run button on a `CodeChunk`), or the containing resource (e.g. presses
    ///   the run button on the parent `Article`).
    ///
    /// - "autorun" means that the resource is automatically executed, without an explicit
    ///   user request do so (but in some cases in response to one).
    ///
    /// ## `Never`
    ///
    /// Never automatically execute the resource.
    /// Only execute when the user explicitly runs the resource (or its containing resource).
    ///
    /// e.g. a user may tag a `CodeBlock` as `@autorun never` if it is long running
    /// and they want to check the outputs of previous code chunks before proceeding
    ///
    /// When generating an execution `Plan`s using:
    ///
    /// - the `PlanOrdering::Topological` option: the resource, and any of its downstream
    ///   dependents should be excluded from the plan.
    ///
    /// - the `PlanOrdering::Appearance` option: the resource, and any following resources
    ///   should be excluded from the plan.
    ///
    /// ## `Needed`
    ///
    /// Execute the resource if it is an upstream dependency of a resource that has been run.
    /// This is the default.
    ///
    /// e.g. `CodeExpression` #1 depends upon a variable assigned in `CodeChunk` #2.
    /// If #2 is run, and #1 is stale, then #1 will be autorun before #2.
    ///
    /// This only affects execution `Plan`s generated with the `PlanOrdering::Topological` option.
    ///
    /// ## `Always`
    ///
    /// Always execute the resource
    ///
    /// e.g. a user may tag a `CodeBlock` as `@autorun always` if it assigns a random variable
    /// (i.e. is non-deterministic) and everytime one of its downstream dependents is run, they
    /// want it to be updated.
    ///
    pub execute_auto: Option<CodeChunkExecuteAuto>,

    /// Whether the resource is marked as pure or impure.
    ///
    /// Pure resources do not modify other resources (i.e. they have no side effects).
    /// This can be determined from whether the resource has any `Declare`, `Assign`, `Alter` or `Write`
    /// in its `relations`. Additionally, the user may mark the resource as pure or impure
    /// either using `@pure` or `@impure` tags in code comments or via user interfaces.
    /// This property stores that explicit mark. If it is `None` then the resources "purity"
    /// will be inferred from its `relations`.
    pub execute_pure: Option<bool>,

    /// The [`ResourceDigest`] of the resource when it was last compiled
    pub compile_digest: Option<ResourceDigest>,

    /// The [`ResourceDigest`] of the resource when it was last executed
    pub execute_digest: Option<ResourceDigest>,

    /// Whether the last execution of the resource succeeded
    pub execute_status: Option<CodeExecutableExecuteStatus>,
}

impl ResourceInfo {
    /// Create a default `ResourceInfo` object with only a reference to a `Resource`
    pub fn default(resource: Resource) -> Self {
        Self {
            resource,
            relations: None,
            dependencies: None,
            dependents: None,
            depth: None,
            execute_auto: None,
            execute_pure: None,
            compile_digest: None,
            execute_digest: None,
            execute_status: None,
        }
    }

    /// Create a new `ResourceInfo` object
    pub fn new(
        resource: Resource,
        relations: Option<Pairs>,
        execute_auto: Option<CodeChunkExecuteAuto>,
        execute_pure: Option<bool>,
        compile_digest: Option<ResourceDigest>,
        execute_digest: Option<ResourceDigest>,
        execute_status: Option<CodeExecutableExecuteStatus>,
    ) -> Self {
        Self {
            resource,
            relations,
            dependencies: None,
            dependents: None,
            depth: None,
            execute_auto,
            execute_pure,
            compile_digest,
            execute_digest,
            execute_status,
        }
    }

    /// Is the resource pure (i.e. has no side effects)?
    ///
    /// If the resource has not been explicitly tagged as pure or impure then
    /// returns `true` if there are no side-effect causing relations.
    pub fn is_pure(&self) -> bool {
        self.execute_pure.unwrap_or_else(|| match &self.relations {
            Some(relations) => !relations.iter().any(|(relation, ..)| {
                matches!(
                    relation,
                    Relation::Declare(..)
                        | Relation::Assign(..)
                        | Relation::Alter(..)
                        | Relation::Import(..)
                        | Relation::Write(..)
                )
            }),
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
                    (Relation::Declare(..), Resource::Symbol(symbol))
                    | (Relation::Assign(..), Resource::Symbol(symbol))
                    | (Relation::Alter(..), Resource::Symbol(symbol)) => Some(symbol),
                    _ => None,
                })
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    /// Is the resource stale?
    ///
    /// Note that, when comparing the `execute_digest` and `compile_digest` for this determination,
    /// the `content_digest` part is ignored. This avoids re-execution in situations such as when
    /// the user removes a `@autorun always` comment (they probably don't want it to be run again
    /// automatically next time). We currently include `dependencies_stale` in the comparison but
    /// that may also be unnecessary/inappropriate as well?
    pub fn is_stale(&self) -> bool {
        if let (Some(compile_digest), Some(execute_digest)) =
            (&self.compile_digest, &self.execute_digest)
        {
            compile_digest.semantic_digest != execute_digest.semantic_digest
                || compile_digest.dependencies_digest != execute_digest.dependencies_digest
                || compile_digest.dependencies_stale != execute_digest.dependencies_stale
        } else {
            true
        }
    }

    /// Did execution fail the last time the resource was executed
    ///
    /// Returns `false` if the resource has not been executed or was executed
    /// and succeeded.
    pub fn is_fail(&self) -> bool {
        matches!(
            self.execute_status,
            Some(CodeExecutableExecuteStatus::Failed)
        )
    }

    /// The resource was executed, so update the `execute_digest` to the `compile_digest`,
    /// and `execute_succeeded` property.
    pub fn did_execute(&mut self, execute_status: Option<CodeExecutableExecuteStatus>) {
        self.execute_digest = self.compile_digest.clone();
        self.execute_status = execute_status;
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
