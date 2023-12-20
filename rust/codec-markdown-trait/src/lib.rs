//! Provides the `MarkdownCodec` trait for generating Markdown for Stencila Schema nodes

use codec_losses::Losses;
use codec_mapping::{Mapping, MappingEntry, NodeId, NodeType};

pub use codec_markdown_derive::MarkdownCodec;
use common::smol_str::SmolStr;

pub trait MarkdownCodec {
    /// Encode a Stencila Schema node to Markdown
    fn to_markdown(&self, context: &mut MarkdownEncodeContext);
}

#[derive(Default)]
pub struct MarkdownEncodeContext {
    /// The encoded Markdown content
    pub content: String,

    /// A stack of node types, ids and start positions
    node_stack: Vec<(NodeType, NodeId, usize)>,

    /// A prefix to prefix all lines with
    escape_char: Option<String>,

    /// A prefix to prefix all lines with
    line_prefix: Vec<String>,

    /// Node to position mapping
    pub mapping: Mapping,

    /// Encoding losses
    pub losses: Losses,

    /// The nesting depth for any node type using fenced divs
    pub depth: usize,

    /// The footnotes for the context
    pub footnotes: Vec<String>,
}

impl MarkdownEncodeContext {
    /// Enter a node
    ///
    /// Pushes the node id and start position onto the stack.
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.node_stack
            .push((node_type, node_id, self.content.len()));
        self
    }

    /// Exit a node
    ///
    /// Pops the node id and start position off the stack and creates a
    /// new mapping entry with those and the current position as end position.
    pub fn exit_node(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let end = self.content.len();
            self.mapping
                .push(MappingEntry::new(start..end, node_type, node_id, None))
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
        self.depth -= 1;
        self
    }

    /// Set the escape chars
    pub fn set_escape(&mut self, char: &str) -> &mut Self {
        self.escape_char = Some(char.to_string());
        self
    }

    /// Clear the escape chars
    pub fn clear_escape(&mut self) -> &mut Self {
        self.escape_char = None;
        self
    }

    /// Add to the line prefix
    pub fn push_line_prefix(&mut self, prefix: &str) -> &mut Self {
        self.line_prefix.push(prefix.to_string());
        self
    }

    /// Clear the line prefix
    pub fn pop_line_prefix(&mut self) -> &mut Self {
        self.line_prefix.pop();
        self
    }

    /// Push a string onto the Markdown content
    pub fn push_str(&mut self, value: &str) -> &mut Self {
        if !self.line_prefix.is_empty() {
            if let Some('\n') = self.content.chars().last() {
                self.content.push_str(&self.line_prefix.join(""));
            }
        }

        if let Some(escape) = &self.escape_char {
            let value = value.replace(escape, &["\\", escape].concat());
            self.content.push_str(&value);
        } else {
            self.content.push_str(value);
        };

        self
    }

    /// Push a property represented as a string onto the Markdown content
    ///
    /// Creates a new mapping entry for the property.
    pub fn push_prop_str(&mut self, prop: &str, value: &str) -> &mut Self {
        let start = self.content.len();

        self.push_str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.content.len();
            self.mapping.push(MappingEntry::new(
                start..end,
                *node_type,
                *node_id,
                Some(SmolStr::from(prop)),
            ));
        }
        self
    }

    /// Push a property by calling a function to push content onto the Markdown
    ///
    /// Creates a new mapping entry for the property.
    pub fn push_prop_fn<F>(&mut self, prop: &str, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        let start = self.content.len();

        func(self);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.content.len();
            self.mapping.push(MappingEntry::new(
                start..end,
                *node_type,
                *node_id,
                Some(SmolStr::from(prop)),
            ));
        }
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
        impl MarkdownCodec for $type {
            fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
                context
                    .push_str(&self.to_string())
                    .add_loss(concat!($name, "@"));
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl MarkdownCodec for String {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.push_str(&self.to_string());
    }
}

impl<T> MarkdownCodec for Box<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        self.as_ref().to_markdown(context)
    }
}

impl<T> MarkdownCodec for Option<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if let Some(value) = self {
            value.to_markdown(context);
        }
    }
}

impl<T> MarkdownCodec for Vec<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        for item in self.iter() {
            item.to_markdown(context);
        }
    }
}
