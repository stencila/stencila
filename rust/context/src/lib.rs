use common::serde::Serialize;
use schema::{CodeChunk, CodeExpression, SoftwareApplication, SoftwareSourceCode, Variable};

/// The context of a document made available to executable nodes
///
/// During execution of a node tree, an `Executor` (see the `node-execute` crate) walks
/// over the tree and collects information about the document into a `Context` object.
/// This context is then made available to [`InstructionBlock`] and [`InstructionInline`]
/// nodes (and potentially others in the future).
///
/// There are currently two groups of properties in this context:
///
/// - Lists of nodes of various types. Intended to be used by specialized assistants
///   that insert or edit nodes of those types to be able to provide examples
///   in their prompts.
///
/// - The `kernels` property which provides information about the state of
///   the document's execution kernels. Intended for use in specialized
///   assistants for `CodeChunk` and `CodeExpression` nodes to help improve
///   the accuracy of generated code.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct Context {
    /// The code chunks in the document
    pub code_chunks: Vec<CodeChunk>,

    /// The code expressions in the document
    pub code_expressions: Vec<CodeExpression>,

    /// Information about the document's execution kernels
    pub kernels: Vec<KernelContext>,
}

/// Contextual information from a kernel
///
/// This encapsulates the information that can be obtained from
/// a `KernelInstance` at runtime.
///
/// Note that `info` and `packages` probably only need to be
/// obtained from a kernel instance once, whereas `variables`
/// needs to be updated whenever a variable is declared or
/// updated in a kernel.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct KernelContext {
    /// Runtime information about the kernel instance
    ///
    /// Obtained from the `KernelInstance::info` method.
    pub info: SoftwareApplication,

    /// A list of packages available in the kernel instance
    ///
    /// Obtained from the `KernelInstance::packages` method.
    pub packages: Vec<SoftwareSourceCode>,

    /// A list of packages available in the kernel instance
    ///
    /// Obtained from the `KernelInstance::packages` method.
    pub variables: Vec<Variable>,
}
