//! Provides the `LatexCodec` trait for generating Latex for Stencila Schema nodes

use std::{env::temp_dir, path::PathBuf};

use codec_info::{EncodeInfo, Losses, Mapping, NodeId, NodeProperty, NodeType};
use common::itertools::Itertools;
use format::Format;

pub use codec_latex_derive::LatexCodec;

/// Encode a node that implements `LatexCodec` to Latex
///
/// A convenience function to save the caller from having to create a context etc.
pub fn to_latex<T>(node: &T, format: Format, standalone: bool, render: bool) -> (String, EncodeInfo)
where
    T: LatexCodec,
{
    let mut context = LatexEncodeContext::new(format, standalone, render);
    node.to_latex(&mut context);

    let mut latex = context.content;
    if latex.ends_with("\n\n") {
        latex.pop();
    }

    let info = EncodeInfo {
        losses: context.losses,
        mapping: context.mapping,
    };

    (latex, info)
}

///
pub fn requires_packages(latex: &str) -> String {
    let mut packages = Vec::new();

    if latex.contains(r"\includegraphics") {
        packages.push("graphicx");
    }

    if latex.contains(r"\landscape") {
        packages.push("pdflscape");
    }

    packages
        .iter()
        .map(|pkg| [r"\usepackage{", pkg, "}"].concat())
        .join("\n")
}

pub trait LatexCodec {
    /// Encode a Stencila Schema node to Latex
    fn to_latex(&self, context: &mut LatexEncodeContext);
}

pub struct LatexEncodeContext {
    /// The format to encode (Latex or Rnw)
    pub format: Format,

    /// Whether the root node should be encoded standalone
    pub standalone: bool,

    /// Whether encoding in render mode (executable outputs)
    pub render: bool,

    /// Whether the root node is "coarse grained" (i.e. decoded with the `--coarse` option).
    /// Used to determine whether newlines are needed between blocks.
    pub coarse: bool,

    /// The encoded Latex content
    pub content: String,

    /// The temporary directory where images are encoded to if necessary
    pub temp_dir: PathBuf,

    /// A stack of node types, ids and start positions
    node_stack: Vec<(NodeType, NodeId, usize)>,

    /// Node to position mapping
    pub mapping: Mapping,

    /// Encoding losses
    pub losses: Losses,

    /// The nesting depth for any node type using fenced divs
    depth: usize,
}

impl LatexEncodeContext {
    pub fn new(format: Format, standalone: bool, render: bool) -> Self {
        let temp_dir = temp_dir();

        Self {
            format,
            standalone,
            render,
            temp_dir,
            coarse: false,
            content: String::default(),
            node_stack: Vec::default(),
            mapping: Mapping::default(),
            losses: Losses::default(),
            depth: 0,
        }
    }

    /// Get the current insertion position (i.e. the number of characters in the content)
    fn char_index(&self) -> usize {
        self.content.chars().count()
    }

    /// Enter a node
    ///
    /// Pushes the node id and start position onto the stack.
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.node_stack
            .push((node_type, node_id, self.char_index()));
        self
    }

    /// Exit a node
    ///
    /// Pops the node id and start position off the stack and creates a
    /// new mapping entry with those and the current position as end position.
    pub fn exit_node(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let mut end = self.char_index();
            if self.content.ends_with("\n\n") {
                end -= 1;
            }
            self.mapping.add(start, end, node_type, node_id, None, None)
        }
        self
    }

    /// Exit the final node
    ///
    /// Should only be used by the top level `Article`. Does not exclude any double newline
    /// at the end from the range.
    pub fn exit_node_final(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let end = self.char_index();
            self.mapping.add(start, end, node_type, node_id, None, None)
        }
        self
    }

    /// Push a property represented by a string content onto the LaTex
    ///
    /// Creates a new mapping entry for the property.
    pub fn property_str(&mut self, prop: NodeProperty, value: &str) -> &mut Self {
        let start = self.char_index();

        self.content.push_str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Push a property by calling a function to push content onto the LaTex
    ///
    /// Creates a new mapping entry for the property.
    pub fn property_fn<F>(&mut self, prop: NodeProperty, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        let start = self.char_index();

        func(self);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Increase the nesting depth
    pub fn increase_depth(&mut self) -> &mut Self {
        self.depth += 1;
        self
    }

    /// Decrease the nesting depth
    pub fn decrease_depth(&mut self) -> &mut Self {
        self.depth = self.depth.saturating_sub(1);
        self
    }

    /// Push a string onto the Latex content
    pub fn str(&mut self, value: &str) -> &mut Self {
        if self.depth > 0 && matches!(self.content.chars().last(), None | Some('\n')) {
            self.content.push_str(&"    ".repeat(self.depth));
        }
        self.content.push_str(value);
        self
    }

    /// Push a character onto the Latex content
    pub fn char(&mut self, value: char) -> &mut Self {
        self.content.push(value);
        self
    }

    /// Add a single space to the end of the content
    pub fn space(&mut self) -> &mut Self {
        self.content.push(' ');
        self
    }

    /// Add a single newline to the end of the content
    pub fn newline(&mut self) -> &mut Self {
        self.content.push('\n');
        self
    }

    /// Enter a LaTeX environment
    pub fn environ_begin(&mut self, name: &str) -> &mut Self {
        self.str("\\begin{");
        self.content.push_str(name);
        self.content.push('}');
        self
    }

    /// Exit a LaTeX environment
    pub fn environ_end(&mut self, name: &str) -> &mut Self {
        self.str("\\end{");
        self.content.push_str(name);
        self.content.push_str("}\n");
        self
    }

    /// Enter a LaTeX command
    pub fn command_enter(&mut self, name: &str) -> &mut Self {
        self.content.push('\\');
        self.content.push_str(name);
        self.content.push('{');
        self
    }

    /// Exit a LaTeX command
    pub fn command_exit(&mut self) -> &mut Self {
        self.content.push('}');
        self
    }

    /// Trim whitespace from the end of the content in-place
    ///
    /// According to [this](https://users.rust-lang.org/t/trim-string-in-place/15809/18)
    /// this is the recommended way to trim in place.
    pub fn trim_end(&mut self) -> &mut Self {
        let trimmed = self.content.trim_end();
        self.content.truncate(trimmed.len());
        self
    }

    /// Add a single loss
    pub fn add_loss(&mut self, label: &str) -> &mut Self {
        self.losses.add(label);
        self
    }

    /// Append a vector of losses
    pub fn merge_losses(&mut self, losses: Losses) -> &mut Self {
        self.losses.merge(losses);
        self
    }
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl LatexCodec for $type {
            fn to_latex(&self, context: &mut LatexEncodeContext) {
                context.str(&self.to_string());
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl LatexCodec for String {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.str(&self.to_string());
    }
}

impl<T> LatexCodec for Box<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        self.as_ref().to_latex(context)
    }
}

impl<T> LatexCodec for Option<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        if let Some(value) = self {
            value.to_latex(context);
        }
    }
}

impl<T> LatexCodec for Vec<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        for item in self.iter() {
            item.to_latex(context);
        }
    }
}
