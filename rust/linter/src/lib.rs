use std::path::Path;

use eyre::Result;
use serde::Serialize;
use strum::Display;

use schema::{AuthorRole, CompilationMessage};

// Re-exports for the convenience of crates implementing the `Linter` trait
pub use async_trait::async_trait;
pub use eyre;
pub use format::Format;
pub use node_type::NodeType;
pub use schema;

#[async_trait]
pub trait Linter: Send + Sync {
    /// The name of the linter
    fn name(&self) -> &str;

    /// The node types supported by the linter
    fn node_types(&self) -> Vec<NodeType>;

    /// The formats supported by the linter
    fn formats(&self) -> Vec<Format>;

    /// The availability of the linter on the current machine
    fn availability(&self) -> LinterAvailability;

    /// Whether the linter support formatting content
    fn supports_formatting(&self) -> bool;

    /// Whether the linter support fixing warning and errors
    fn supports_fixing(&self) -> bool;

    /// Lint some content at a path
    async fn lint(
        &self,
        content: &str,
        path: &Path,
        options: &LintingOptions,
    ) -> Result<LintingOutput>;
}

/// The availability of a linter on the current machine
#[derive(Debug, Display, Clone, Copy, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum LinterAvailability {
    /// Available on this machine
    Available,
    /// Available on this machine but requires installation
    Installable,
    /// Not available on this machine
    Unavailable,
}

/// A linter specification
///
/// Used to serialize the list of available linters
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinterSpecification {
    node_types: Vec<NodeType>,
    formats: Vec<Format>,
    availability: LinterAvailability,
    supports_formatting: bool,
    supports_fixing: bool,
}

impl From<&dyn Linter> for LinterSpecification {
    fn from(linter: &dyn Linter) -> Self {
        Self {
            node_types: linter.node_types(),
            formats: linter.formats(),
            availability: linter.availability(),
            supports_formatting: linter.supports_formatting(),
            supports_fixing: linter.supports_fixing(),
        }
    }
}

/// Options for linting
#[derive(Debug, Default)]
pub struct LintingOptions {
    /// The name of the linter to use
    ///
    /// If not specified then all linters that support the node type and format
    /// will the applied.
    pub linter: Option<String>,

    /// The node type being linted
    pub node_type: Option<NodeType>,

    /// The format of the content being linted
    pub format: Option<Format>,

    /// Whether to format the code
    pub should_format: bool,

    /// Whether to fix the code
    pub should_fix: bool,
}

/// Output from linting
#[derive(Default)]
pub struct LintingOutput {
    /// Any diagnostic messages
    ///
    /// The output from linting tool/s parsed to compilation messages.
    /// The can be used for displaying diagnostic messages at the correct line/column.
    ///
    /// Will usually, but not necessarily, be `None` if `output` is `Some`.
    /// Implementations should return `None` rather than an empty vector.
    pub messages: Option<Vec<CompilationMessage>>,

    /// Any software authors that contributed to the linting
    ///
    /// The `role_name` of these authors should be `Formatter` or `Linter`.
    pub authors: Option<Vec<AuthorRole>>,

    /// The formatted and/or fixed content
    ///
    /// If both `format` and `fix` are false, or if there is no change in the content,
    /// this is expected to be `None`
    pub content: Option<String>,
}
