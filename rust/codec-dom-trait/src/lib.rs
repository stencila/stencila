//! Provides the `DomCodec` trait for generating HTML for the
//! browser DOM for Stencila Schema nodes

use std::collections::HashMap;

use common::{
    inflector::Inflector, itertools::Itertools, once_cell::sync::Lazy, regex::Regex,
    serde::Serialize, serde_json, smart_default::SmartDefault,
};
use html_escape::{encode_safe, encode_single_quoted_attribute};
use node_id::NodeId;
use node_type::NodeType;

pub use codec_dom_derive::DomCodec;

pub trait DomCodec {
    /// Encode a Stencila Schema node to DOM HTML
    fn to_dom(&self, context: &mut DomEncodeContext);

    /// Encode a Stencila Schema node to a HTML attribute value
    ///
    /// Will generally only be used on simple types (e.g. numbers,
    /// strings, arrays of strings).
    ///
    /// This default implementation serializes the node to JSON.
    /// Should only need to be overridden by atomic types (for perf)
    /// and `String` (to avoid unnecessary double quotes).
    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext)
    where
        Self: Serialize,
    {
        context.push_attr(name, &serde_json::to_string(self).unwrap_or_default());
    }
}

macro_rules! atom {
    ($type:ty, $name:literal) => {
        impl DomCodec for $type {
            fn to_dom(&self, context: &mut DomEncodeContext) {
                context
                    .enter_elem(concat!("stencila-", $name))
                    .push_text(&self.to_string())
                    .exit_elem();
            }

            fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
                context.push_attr(name, &self.to_string());
            }
        }
    };
}

atom!(bool, "boolean");
atom!(i64, "integer");
atom!(u64, "unsigned-integer");
atom!(f64, "number");

impl DomCodec for String {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_elem("stencila-string")
            .push_text(self)
            .exit_elem();
    }

    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
        context.push_attr(name, self);
    }
}

impl<T> DomCodec for Box<T>
where
    T: DomCodec,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        self.as_ref().to_dom(context)
    }
}

impl<T> DomCodec for Option<T>
where
    T: DomCodec + Serialize,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        if let Some(value) = self {
            value.to_dom(context);
        }
    }

    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
        if let Some(value) = self {
            value.to_dom_attr(name, context);
        }
    }
}

impl<T> DomCodec for Vec<T>
where
    T: DomCodec,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        for item in self.iter() {
            item.to_dom(context);
        }
    }
}

#[derive(SmartDefault)]
pub struct DomEncodeContext {
    /// The DOM HTML content
    content: String,

    /// The node type of ancestors of the current node
    node_types: Vec<NodeType>,

    /// The names of the current stack of HTML elements
    elements: Vec<String>,

    /// The levels and ids of the current stack of `Heading` nodes
    headings: Vec<(i64, NodeId)>,

    /// The CSS classes in the document
    css: HashMap<String, String>,

    /// Whether encoding to a standalone document
    pub standalone: bool,

    /// The maximum number of rows of a datatable to encode
    #[default = 1000]
    pub max_datatable_rows: usize,
}

impl DomEncodeContext {
    pub fn new(standalone: bool) -> Self {
        Self {
            standalone,
            ..Default::default()
        }
    }

    /// Enter an element
    pub fn enter_elem(&mut self, name: &str) -> &mut Self {
        self.content.push('<');
        self.content.push_str(name);
        self.content.push('>');

        self.elements.push(name.to_string());

        self
    }

    /// Enter an element with one or more attributes
    ///
    /// Optimized equivalent of `enter_elem(name).push(attr, value).push(...)`
    pub fn enter_elem_attrs<const N: usize>(
        &mut self,
        name: &str,
        attrs: [(&str, &str); N],
    ) -> &mut Self {
        self.content.push('<');
        self.content.push_str(name);
        for (attr, value) in attrs {
            self.content.push(' ');
            self.content.push_str(attr);
            self.content.push('=');
            self.push_attr_value(value);
        }
        self.content.push('>');

        self.elements.push(name.to_string());

        self
    }

    /// Enter an element for a node
    pub fn enter_node_elem(
        &mut self,
        name: &str,
        node_type: NodeType,
        node_id: NodeId,
    ) -> &mut Self {
        let id = node_id.to_string();
        let depth = self.node_types.len().to_string();
        let ancestors = self
            .node_types
            .iter()
            .map(|node_type| node_type.to_string())
            .join(".");

        self.enter_elem_attrs(
            name,
            [("id", &id), ("depth", &depth), ("ancestors", &ancestors)],
        );
        self.node_types.push(node_type);

        self
    }

    /// Enter a node with the default, custom element for the node type
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        let name = ["stencila-", &node_type.to_string().to_kebab_case()].concat();
        self.enter_node_elem(&name, node_type, node_id)
    }

    /// Enter a heading by adding `<stencila-heading-end>` custom elements for
    /// any previous elements that have a level equal to or greater than the
    /// heading being entered into
    pub fn enter_heading(&mut self, level: i64, node_id: NodeId) -> &mut Self {
        while let Some((prev_level, ..)) = self.headings.last() {
            if prev_level < &level {
                break;
            }

            let (.., node_id) = self.headings.pop().expect("checked in parent if");
            self.enter_elem_attrs("stencila-heading-end", [("heading", &node_id.to_string())])
                .exit_elem();
        }

        self.headings.push((level, node_id.clone()));

        self.enter_node(NodeType::Heading, node_id)
    }

    /// Push an attribute onto the current element
    pub fn push_attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.content.pop();

        self.content.push(' ');
        self.content.push_str(name);
        self.content.push('=');
        self.push_attr_value(value);
        self.content.push('>');

        self
    }

    /// Push an attribute value onto the current element
    fn push_attr_value(&mut self, value: &str) {
        if value.is_empty() || value.contains(['"', '\'', ' ', '\t', '\n', '>', '<']) {
            // Use single quoting (more terse for JSON attributes) only if necessary
            let escaped = encode_single_quoted_attribute(value);
            self.content.push('\'');
            self.content.push_str(&escaped);
            self.content.push('\'');
        } else {
            // Value does not contain quotes etc so does not need quoting
            self.content.push_str(value)
        }
    }

    /// Push the `id`` property of `Entity` nodes
    ///
    /// Uses `@id` as the attribute to avoid conflict with `id` which is
    /// necessary for DOM syncing etc
    pub fn push_id(&mut self, id: &Option<String>) -> &mut Self {
        if let Some(id) = id {
            self.push_attr("@id", id);
        }

        self
    }

    /// Push unescaped HTML onto the content
    pub fn push_html(&mut self, html: &str) -> &mut Self {
        self.content.push_str(html);

        self
    }

    /// Push text onto the content
    pub fn push_text(&mut self, text: &str) -> &mut Self {
        let escaped = encode_safe(text);
        self.content.push_str(&escaped);

        self
    }

    /// Push CSS classes onto the context
    pub fn push_css(&mut self, css: &str) -> &mut Self {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\.([\w-]+)\{([^}]+)\}").expect("invalid regex"));

        for captures in REGEX.captures_iter(css) {
            let class = &captures[1];
            let rules = &captures[2];
            if !self.css.contains_key(class) {
                self.css.insert(class.to_string(), rules.to_string());
            }
        }

        self
    }

    /// Enter a slot child element
    pub fn enter_slot(&mut self, tag: &str, name: &str) -> &mut Self {
        self.enter_elem_attrs(tag, [("slot", &name.to_kebab_case())])
    }

    /// Exit a slot child element
    pub fn exit_slot(&mut self) -> &mut Self {
        self.exit_elem()
    }

    /// Push a property as a slotted child element
    pub fn push_slot_fn<F>(&mut self, tag: &str, name: &str, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        self.enter_slot(tag, name);
        func(self);
        self.exit_slot()
    }

    /// Exit a element
    pub fn exit_elem(&mut self) -> &mut Self {
        if let Some(name) = self.elements.pop() {
            self.content.push_str("</");
            self.content.push_str(&name);
            self.content.push('>');
        };

        self
    }

    /// Exit a node
    pub fn exit_node(&mut self) -> &mut Self {
        self.exit_elem();
        self.node_types.pop();

        self
    }

    /// Get the content of the encoding context at completion of encoding
    pub fn content(&mut self) -> String {
        // Use take instead of cloning for performance
        std::mem::take(&mut self.content)
    }

    /// Get a CSS <style> element for the document at completion of encoding
    /// 
    /// This should be placed in the <head> if standalone (to avoid flash of unstyled content),
    /// or at the top of the root node otherwise.
    pub fn style(&self) -> String {
        if !self.css.is_empty() {
            let mut style = "<style>".to_string();
            for (class, css) in self.css.iter() {
                style.push('.');
                style.push_str(class);
                style.push('{');
                style.push_str(css);
                style.push('}');
            }
            style += "</style>";
            style
        } else {
            String::new()
        }
    }
}
