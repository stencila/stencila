use std::collections::VecDeque;

use common::{serde::Serialize, smart_default::SmartDefault};
use schema::{
    CodeChunk, CodeExpression, Heading, InstructionBlock, InstructionInline, MathBlock, MathInline,
    Paragraph, SoftwareApplication, SoftwareSourceCode, StyledBlock, StyledInline, Text, Variable,
};

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
#[derive(Debug, SmartDefault, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct Context {
    /// The title of the document
    pub title: Option<String>,

    /// The genre of the document
    pub genre: Option<String>,

    /// The keywords of the document
    pub keywords: Vec<String>,

    /// The code chunks in the document prior to the current node
    pub code_chunks: Vec<CodeChunk>,

    /// The code expressions in the document prior to the current node
    pub code_expressions: Vec<CodeExpression>,

    /// The instruction blocks in the document prior to the current node
    pub instruction_blocks: Vec<InstructionBlock>,

    /// The instruction inlines in the document prior to the current node
    pub instruction_inlines: Vec<InstructionInline>,

    /// The math blocks in the document prior to the current node
    pub math_blocks: Vec<MathBlock>,

    /// The math inlines in the document prior to the current node
    pub math_inlines: Vec<MathInline>,

    /// The headings in the document prior to the current node
    pub headings: Vec<Heading>,

    /// The paragraphs in the document prior to the current node
    pub paragraphs: Vec<Paragraph>,

    /// The styled blocks in the document prior to the current node
    pub styled_blocks: Vec<StyledBlock>,

    /// The styled inlines in the document prior to the current node
    pub styled_inlines: Vec<StyledInline>,

    /// Text fragments in the document immediately prior to the current node
    pub texts: VecDeque<String>,

    /// The maximum number of text fragments to store when waling over a document
    #[default = 10]
    pub texts_length: usize,

    /// Information about the document's execution kernels
    pub kernels: Vec<KernelContext>,
}

impl Context {
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn set_genre(&mut self, genre: &str) {
        self.genre = Some(genre.to_string());
    }

    pub fn set_keywords(&mut self, keywords: &[String]) {
        self.keywords = keywords.to_vec();
    }

    pub fn push_code_chunk(&mut self, code_chunk: &CodeChunk) {
        self.code_chunks.push(code_chunk.clone());
    }

    pub fn push_code_expression(&mut self, code_expression: &CodeExpression) {
        self.code_expressions.push(code_expression.clone());
    }

    pub fn push_instruction_block(&mut self, instruction_block: &InstructionBlock) {
        self.instruction_blocks.push(instruction_block.clone());
    }

    pub fn push_instruction_inline(&mut self, instruction_inline: &InstructionInline) {
        self.instruction_inlines.push(instruction_inline.clone());
    }

    pub fn push_math_block(&mut self, math_block: &MathBlock) {
        self.math_blocks.push(math_block.clone());
    }

    pub fn push_math_inline(&mut self, math_inline: &MathInline) {
        self.math_inlines.push(math_inline.clone());
    }

    pub fn push_heading(&mut self, heading: &Heading) {
        self.headings.push(heading.clone());
    }

    pub fn push_paragraph(&mut self, paragraph: &Paragraph) {
        self.paragraphs.push(paragraph.clone());
    }

    pub fn push_styled_block(&mut self, styled_block: &StyledBlock) {
        self.styled_blocks.push(styled_block.clone());
    }

    pub fn push_styled_inline(&mut self, styled_inline: &StyledInline) {
        self.styled_inlines.push(styled_inline.clone());
    }

    pub fn push_text(&mut self, text: &Text) {
        self.texts.push_back(text.to_value_string());
        if self.texts.len() > self.texts_length {
            self.texts.pop_front();
        }
    }
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
