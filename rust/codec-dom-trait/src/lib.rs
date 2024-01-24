//! Provides the `DomCodec` trait for generating HTML for the
//! browser DOM for Stencila Schema nodes

use common::{inflector::Inflector, serde::Serialize, serde_json};
use html_escape::{encode_safe, encode_single_quoted_attribute};
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
        context.push_text(&self);
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

#[derive(Default)]
pub struct DomEncodeContext {
    // The DOM HTML content
    content: String,

    // The names of the current stack of HTML elements
    elements: Vec<String>,
}

impl DomEncodeContext {
    /// Enter a node
    pub fn enter_node(&mut self, node_type: NodeType) -> &mut Self {
        let name = ["stencila-", &node_type.to_string().to_kebab_case()].concat();
        self.enter_elem(&name)
    }

    /// Enter a element
    pub fn enter_elem(&mut self, name: &str) -> &mut Self {
        self.content.push('<');
        self.content.push_str(name);
        self.content.push('>');

        self.elements.push(name.to_string());

        self
    }

    /// Push an attribute onto the current element
    pub fn push_attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.content.pop();

        self.content.push(' ');
        self.content.push_str(name);
        self.content.push('=');

        if value.contains(['"', '\'', ' ', '\t', '\n', '>', '<']) {
            // Use single quoting (more terse for JSON attributes) only if necessary
            let escaped = encode_single_quoted_attribute(value);
            self.content.push('\'');
            self.content.push_str(&escaped);
            self.content.push('\'');
        } else {
            // Value does not contain quotes etc so does not need quoting
            self.content.push_str(value)
        }

        self.content.push('>');

        self
    }

    /// Push a slot attribute
    pub fn push_slot(&mut self, name: &str) -> &mut Self {
        self.push_attr("slot", &name.to_kebab_case())
    }

    /// Push text onto the content
    pub fn push_text(&mut self, text: &str) -> &mut Self {
        let escaped = encode_safe(text);
        self.content.push_str(&escaped);

        self
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
        self.exit_elem()
    }

    /// Get the content of the encoding context at completion of encoding
    pub fn content(self) -> String {
        self.content
    }
}
