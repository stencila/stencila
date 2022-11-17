use std::path::{Path, PathBuf};

use schemars::JsonSchema;

use common::{
    derivative::Derivative,
    eyre::Result,
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{self, Serialize},
    serde_with::skip_serializing_none,
};
use formats::Format;
use path_utils::path_slash::PathExt;
use stencila_schema::{ExecutionAuto, ExecutionDigest};

use crate::{execution_digest_from_path, Pairs, Relation};

/// A resource in a dependency graph (the nodes of the graph)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
#[serde(tag = "type", crate = "common::serde")]
pub enum Resource {
    /// A symbol within code, within a document
    Symbol(Symbol),

    /// A node containing code, or associated with code, within a document
    Code(Code),

    /// A node within a document
    Node(Node),

    /// A file on the local filesystem
    File(File),

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
            Resource::Module(Module { language, name, .. }) => {
                ["module://", &language.to_string(), "#", name].concat()
            }
            Resource::Url(Url { url }) => url.clone(),
        }
    }

    /// Generate a [`ResourceDigest`] for a resource.
    ///
    /// If the resource variant does not support generation of a digest,
    /// a default (empty) digest is returned.
    pub fn execution_digest(&self) -> ExecutionDigest {
        match self {
            Resource::File(File { path }) => execution_digest_from_path(path, None),
            _ => ExecutionDigest::default(),
        }
    }

    /// Get the [`ResourceInfo`] for a resource
    pub fn resource_info(&self) -> ResourceInfo {
        ResourceInfo::new(
            self.clone(),
            None,
            None,
            None,
            None,
            Some(self.execution_digest()),
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

/// A tag declared in a `CodeChunk`
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(crate = "common::serde")]
pub struct Tag {
    /// The name of the tag e.g. `uses`, `db`
    pub name: String,

    /// The value of the tag
    pub value: String,

    /// Whether the tag is global to the containing document
    pub global: bool,
}

/// A collection of tags
///
/// Implements a `HashMap` like interface but is implemented as a `Vec` as this
/// is expected to be more performant (in memory and CPU) given that the number
/// of tags in a `TagMap` will usually be small (<10).
#[derive(Debug, Default, Clone, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct TagMap {
    inner: Vec<Tag>,
}

impl TagMap {
    /// Create a new tag map from a list of name/value pairs
    pub fn from_name_values(pairs: &[(&str, &str)]) -> Self {
        let mut map = Self::default();
        for (name, value) in pairs {
            map.insert(Tag {
                name: name.to_string(),
                value: value.to_string(),
                ..Default::default()
            });
        }
        map
    }

    /// Get a tag by name
    pub fn get(&self, name: &str) -> Option<&Tag> {
        self.inner.iter().find(|tag| tag.name == name)
    }

    /// Get a tag value by name
    pub fn get_value(&self, name: &str) -> Option<String> {
        self.get(name).map(|tag| tag.value.clone())
    }

    /// Get a tag split into individual space or comma separated items
    pub fn get_items(&self, name: &str) -> Vec<String> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

        match self.get_value(name) {
            Some(value) => REGEX.split(&value).map(String::from).collect_vec(),
            None => Vec::new(),
        }
    }

    /// Insert a tag
    ///
    /// Overrides any existing tag with the same `name`.
    pub fn insert(&mut self, new: Tag) {
        if let Some((position, ..)) = self.inner.iter().find_position(|tag| tag.name == new.name) {
            self.inner[position] = new;
        } else {
            self.inner.push(new)
        }
    }

    /// Insert `global` tags from another tag map
    ///
    /// Used to merge a resource's global tags into a document's global tags.
    pub fn insert_globals(&mut self, other: &TagMap) {
        for tag in other.inner.iter() {
            if tag.global {
                self.insert(tag.clone());
            }
        }
    }

    /// Merge tags from one tag map into another, overriding any duplicates
    ///
    /// Used to merge document's global tags into a resource's tags.
    pub fn merge(&self, other: &TagMap) -> TagMap {
        let mut clone = self.clone();
        for tag in &other.inner {
            clone.insert(tag.clone());
        }
        clone
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
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
    /// ## `WhenNecessary`
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
    /// e.g. a user may tag a `CodeChunk` as `@autorun always` if it assigns a random variable
    /// (i.e. is non-deterministic) and everytime one of its downstream dependents is run, they
    /// want it to be updated.
    ///
    pub execute_auto: Option<ExecutionAuto>,

    /// Whether the resource is marked as pure or impure.
    ///
    /// Pure resources do not modify other resources (i.e. they have no side effects).
    /// This can be determined from whether the resource has any `Declare`, `Assign`, `Alter` or `Write`
    /// in its `relations`. Additionally, the user may mark the resource as pure or impure
    /// either using `@pure` or `@impure` tags in code comments or via user interfaces.
    /// This property stores that explicit mark. If it is `None` then the resources "purity"
    /// will be inferred from its `relations`.
    pub execute_pure: Option<bool>,

    /// For code resources, whether the code had syntax errors when it was last parsed
    pub syntax_errors: Option<bool>,

    /// The [`ExecutionDigest`] of the resource when it was last compiled
    pub compile_digest: Option<ExecutionDigest>,

    /// The [`ExecutionDigest`] of the resource when it was last executed
    pub execute_digest: Option<ExecutionDigest>,

    /// Whether the last execution of the resource failed or not
    ///
    /// Used to determine if other resources should have `execution_required` set to `DependenciesFailed`.
    /// Should be false if the resource has never executed or succeeded last time it was.
    pub execute_failed: Option<bool>,

    /// The tags defined in the resource (if it is a `CodeChunk`)
    pub tags: TagMap,
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
            syntax_errors: None,
            compile_digest: None,
            execute_digest: None,
            execute_failed: None,
            tags: TagMap::default(),
        }
    }

    /// Create a new `ResourceInfo` object
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        resource: Resource,
        relations: Option<Pairs>,
        execute_auto: Option<ExecutionAuto>,
        execute_pure: Option<bool>,
        syntax_errors: Option<bool>,
        compile_digest: Option<ExecutionDigest>,
        execute_digest: Option<ExecutionDigest>,
        execute_failed: Option<bool>,
    ) -> Self {
        Self {
            resource,
            relations,
            dependencies: None,
            dependents: None,
            depth: None,
            execute_auto,
            execute_pure,
            syntax_errors,
            compile_digest,
            execute_digest,
            execute_failed,
            tags: TagMap::default(),
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
                    Relation::Declares(..)
                        | Relation::Assigns(..)
                        | Relation::Alters(..)
                        | Relation::Imports(..)
                        | Relation::Writes(..)
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
                    (Relation::Uses(..), Resource::Symbol(symbol)) => Some(symbol),
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
                    (Relation::Declares(..), Resource::Symbol(symbol))
                    | (Relation::Assigns(..), Resource::Symbol(symbol))
                    | (Relation::Alters(..), Resource::Symbol(symbol)) => Some(symbol),
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
    pub fn is_fail(&self) -> bool {
        if let Some(failed) = self.execute_failed {
            failed
        } else {
            false
        }
    }

    /// The resource was executed, so update the `execute_digest` to the `compile_digest`,
    /// and `execute_succeeded` property.
    pub fn did_execute(&mut self, execute_failed: bool) {
        self.execute_digest = self.compile_digest.clone();
        self.execute_failed = Some(execute_failed);
    }
}

/// A change to a resource
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub struct ResourceChange {
    pub resource: Resource,
    pub action: ResourceChangeAction,
    pub time: String,
}

/// The type of change to a resource
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub enum ResourceChangeAction {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Derivative, JsonSchema, Serialize)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(crate = "common::serde")]
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
#[serde(crate = "common::serde")]
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
#[serde(crate = "common::serde")]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[schemars(deny_unknown_fields)]
pub struct Code {
    /// The path of the file that the node is defined in
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,

    /// The id of the node with the document
    pub id: String,

    /// The type of node e.g. `Parameter`, `CodeChunk`, `Call`
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    pub kind: String,

    /// The programming language associated with the node
    ///
    /// Can be [`Format::Unknown`].
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    pub language: Format,
}

/// Create a new `Executable` resource
pub fn code(path: &Path, id: &str, kind: &str, language: Format) -> Resource {
    Resource::Code(Code {
        path: path.to_path_buf(),
        id: id.into(),
        kind: kind.into(),
        language,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
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
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Module {
    /// The programming language of the module
    pub language: Format,

    /// The name of the module
    pub name: String,
}

/// Create a new `Module` resource
pub fn module(language: Format, name: &str) -> Resource {
    Resource::Module(Module {
        language,
        name: name.into(),
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
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
