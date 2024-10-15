//! Provides the `MarkdownCodec` trait for generating Markdown for Stencila Schema nodes

use codec_info::{Losses, Mapping, MessageLevel, NodeId, NodeProperty, NodeType};
use common::{inflector::Inflector, tracing};
use format::Format;

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
    /// The format to render to
    pub format: Format,

    /// The render option of `codec::EncodeOptions`
    pub render: bool,

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
    pub fn new(format: Option<Format>, render: Option<bool>) -> Self {
        Self {
            format: format.unwrap_or(Format::Smd), // Default to Stencila Markdown
            render: render.unwrap_or_default(),
            ..Default::default()
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
            // Do not include any blank line after the node in the range
            // for the node
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

    /// Push the colons for a fenced div onto the content
    pub fn push_colons(&mut self) -> &mut Self {
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

    /// Push a string with authorship info onto the Markdown content
    pub fn push_authored_str(&mut self, runs: &[(u8, u64, u8, u32)], value: &str) -> &mut Self {
        let mut start = self.char_index();

        self.push_str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            for &(count, authors, mii, length) in runs {
                self.mapping.add(
                    start,
                    start + (length as usize),
                    *node_type,
                    node_id.clone(),
                    None,
                    Some((count, authors, mii)),
                );
                start += length as usize
            }
        }

        self
    }

    /// Push a property represented as a string onto the Markdown content
    ///
    /// Creates a new mapping entry for the property.
    pub fn push_prop_str(&mut self, prop: NodeProperty, value: &str) -> &mut Self {
        let start = self.char_index();

        self.push_str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
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
        let start = self.char_index();

        func(self);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Create a MyST directive
    ///
    /// See spec at https://mystmd.org/spec/overview#directives
    pub fn myst_directive<A, O, C>(
        &mut self,
        fence: char,
        name: &str,
        args: A,
        options: O,
        content: C,
    ) -> &mut Self
    where
        A: Fn(&mut Self),
        O: Fn(&mut Self),
        C: Fn(&mut Self),
    {
        // Opening fence
        self.content.push_str(
            &fence
                .to_string()
                .repeat(3 + self.depth * if fence == '`' { 1 } else { 2 }),
        );

        // Name
        self.content.push('{');
        self.content.push_str(name);
        self.content.push('}');

        // Args
        args(self);

        self.newline();

        // Add options, and if any, or if semicolon fence, then add a separating line
        let start = self.char_index();
        options(self);
        if fence == ':' || self.char_index() != start {
            self.newline();
        }

        // Content
        self.increase_depth();
        content(self);
        self.decrease_depth();

        // Closing fence
        self.content.push_str(
            &fence
                .to_string()
                .repeat(3 + self.depth * if fence == '`' { 1 } else { 2 }),
        );
        self.content.push('\n');

        self
    }

    /// Push a property represented as a MyST directive option
    ///
    /// Write a line to the file with a kebab-cased property name and creates a new mapping
    /// entry for the property.
    pub fn myst_directive_option(
        &mut self,
        prop: NodeProperty,
        name: Option<&str>,
        value: &str,
    ) -> &mut Self {
        let name = name
            .map(String::from)
            .unwrap_or_else(|| prop.to_string().to_kebab_case());

        self.push_str(":")
            .push_str(&name)
            .push_str(": ")
            .push_prop_str(prop, value)
            .newline()
    }

    /// Create a MyST role
    ///
    /// See spec at https://mystmd.org/spec/overview#roles
    pub fn myst_role<C>(&mut self, name: &str, content: C) -> &mut Self
    where
        C: Fn(&mut Self),
    {
        self.content.push('{');
        self.content.push_str(name);
        self.content.push_str("}`");

        content(self);

        self.content.push('`');
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
                    None,
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

    /// Add a message to the context
    ///
    /// Currently this only logs messages but in the future may return them
    /// as part of `EncodeInfo` (as is done for `DecodeInfo`)
    pub fn add_message(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        level: MessageLevel,
        message: String,
    ) {
        let message = format!("{node_type} {node_id}: {message}");
        match level {
            MessageLevel::Trace => tracing::trace!(message),
            MessageLevel::Debug => tracing::debug!(message),
            MessageLevel::Info => tracing::info!(message),
            MessageLevel::Warning => tracing::warn!(message),
            MessageLevel::Error => tracing::error!(message),
        }
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
