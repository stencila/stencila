//! Provides the `MarkdownCodec` trait for generating Markdown for Stencila Schema nodes

use codec_info::{Losses, Mapping, NodeId, NodeProperty, NodeType};

pub use codec_markdown_derive::MarkdownCodec;

pub trait MarkdownCodec {
    /// Encode a Stencila Schema node to Markdown
    fn to_markdown(&self, context: &mut MarkdownEncodeContext);
}

/// Encode a node that implements `MarkdownCodec` to Markdown
///
/// A convenience function to save the caller from having to create a context etc.
pub fn to_markdown<T>(node: &T) -> String
where
    T: MarkdownCodec,
{
    let mut context = MarkdownEncodeContext::default();
    node.to_markdown(&mut context);
    context.content.trim().to_string()
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

    /// Should empty lines be prefixed?
    prefix_empty_lines: bool,

    /// Node to position mapping
    pub mapping: Mapping,

    /// Encoding losses
    pub losses: Losses,

    /// The nesting depth for any node type using fenced divs
    pub depth: usize,

    /// The footnotes for the context
    pub footnotes: Vec<Self>,
}

impl MarkdownEncodeContext {
    /// Get the current insertion position (i.e. the number of characters in the content)
    fn position(&self) -> usize {
        self.content.chars().count()
    }

    /// Enter a node
    ///
    /// Pushes the node id and start position onto the stack.
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.node_stack.push((node_type, node_id, self.position()));
        self
    }

    /// Exit a node
    ///
    /// Pops the node id and start position off the stack and creates a
    /// new mapping entry with those and the current position as end position.
    pub fn exit_node(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let mut end = self.position();
            // Do not include any blank line after the node in the range
            // for the node
            if self.content.ends_with("\n\n") {
                end -= 1;
            }
            self.mapping.add(start, end, node_type, node_id, None)
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

    /// Get the semicolons for a fenced div
    pub fn push_semis(&mut self) -> &mut Self {
        self.content.push_str(&":".repeat(3 + self.depth * 2));
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

    /// Set whether to prefix empty lines
    pub fn prefix_empty_lines(&mut self, yes_no: bool) -> &mut Self {
        self.prefix_empty_lines = yes_no;
        self
    }

    /// Push a string onto the Markdown content
    pub fn push_str(&mut self, value: &str) -> &mut Self {
        let value = if !self.line_prefix.is_empty() {
            let prefix = self.line_prefix.join("");

            // If at start of content or last char is a newline add line prefix
            if matches!(self.content.chars().last(), None | Some('\n'))
                && ((value.starts_with('\n') && self.prefix_empty_lines)
                    || !value.starts_with('\n'))
            {
                self.content.push_str(&prefix);
            }

            // If content contains inner empty lines then ensure those are prefixed too
            if self.prefix_empty_lines {
                value.replace("\n\n", &["\n", &prefix, "\n"].concat())
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };

        if let Some(escape) = &self.escape_char {
            let value = value.replace(escape, &["\\", escape].concat());
            self.content.push_str(&value);
        } else {
            self.content.push_str(&value);
        };

        self
    }

    /// Push a property represented as a string onto the Markdown content
    ///
    /// Creates a new mapping entry for the property.
    pub fn push_prop_str(&mut self, prop: NodeProperty, value: &str) -> &mut Self {
        let start = self.position();

        self.push_str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.position();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop));
        }
        self
    }

    /// Push a property by calling a function to push content onto the Markdown
    ///
    /// Creates a new mapping entry for the property.
    pub fn push_prop_fn<F>(&mut self, prop: NodeProperty, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        let start = self.position();

        func(self);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.position();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop));
        }
        self
    }

    /// A a single newline to the end of the content
    pub fn newline(&mut self) -> &mut Self {
        self.content.push('\n');
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

    /// Trim the end matches of the content in-place
    pub fn trim_end_matches<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(char) -> bool,
    {
        let trimmed = self.content.trim_end_matches(func);
        self.content.truncate(trimmed.len());
        self
    }

    /// Append footnotes to the end of the content
    pub fn append_footnotes(&mut self) {
        for footnote in self.footnotes.drain(..) {
            let offset = self.content.chars().count();
            for entry in footnote.mapping.entries() {
                self.mapping.add(
                    offset + entry.range.start,
                    offset + entry.range.end,
                    entry.node_type,
                    entry.node_id.clone(),
                    entry.property,
                );
            }
            self.content.push_str(&footnote.content);
        }
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
