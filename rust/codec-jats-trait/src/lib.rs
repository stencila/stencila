//! Provides the `JatsCodec` trait for generating JATS XML for Stencila Schema nodes.

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use stencila_codec_info::Losses;
pub use stencila_codec_jats_derive::JatsCodec;

pub mod encode;
use encode::escape;

/// Encode a value to JATS XML.
pub trait JatsCodec {
    /// Append the JATS representation of the value to the encoding context.
    fn to_jats(&self, context: &mut JatsEncodeContext);
}

/// Encode a value to a JATS fragment.
pub fn to_jats<T>(value: &T) -> Result<(String, Losses), JatsEncodeError>
where
    T: JatsCodec + ?Sized,
{
    let mut context = JatsEncodeContext::default();
    value.to_jats(&mut context);
    context.finish()
}

#[derive(Debug)]
struct Element {
    name: String,
    start: usize,
    content_start: Option<usize>,
}

/// State accumulated while encoding a JATS document or fragment.
#[derive(Default)]
pub struct JatsEncodeContext {
    content: String,
    elements: Vec<Element>,
    losses: Losses,
    error: Option<JatsEncodeError>,
    reference_ids: HashMap<String, String>,
    used_reference_ids: HashSet<String>,
}

impl JatsEncodeContext {
    /// Enter an XML element.
    pub fn enter_elem(&mut self, name: &str) -> &mut Self {
        self.close_start_tag();

        let start = self.content.len();
        self.content.push('<');
        self.content.push_str(name);
        self.elements.push(Element {
            name: name.to_string(),
            start,
            content_start: None,
        });
        self
    }

    /// Add an attribute to the element most recently entered.
    pub fn push_attr(&mut self, name: &str, value: impl fmt::Display) -> &mut Self {
        if self
            .elements
            .last()
            .is_none_or(|element| element.content_start.is_some())
        {
            self.set_error(JatsEncodeError::AttributeOutsideStartTag(name.to_string()));
            return self;
        }

        self.content.push(' ');
        self.content.push_str(name);
        self.content.push_str("=\"");
        self.content.push_str(&escape(value.to_string()));
        self.content.push('"');
        self
    }

    /// Append escaped XML text.
    pub fn push_text(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.close_start_tag();
        self.content.push_str(&escape(value));
        self
    }

    /// Append trusted, already encoded XML.
    pub fn push_xml(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.close_start_tag();
        self.content.push_str(value.as_ref());
        self
    }

    /// Exit the element most recently entered.
    pub fn exit_elem(&mut self) -> &mut Self {
        let Some(element) = self.elements.pop() else {
            self.set_error(JatsEncodeError::ElementStackUnderflow);
            return self;
        };

        if element.content_start.is_none() {
            self.content.push('>');
        }
        self.content.push_str("</");
        self.content.push_str(&element.name);
        self.content.push('>');
        self
    }

    /// Exit the most recently entered element, removing it when it has no content.
    pub fn exit_elem_omit_empty(&mut self) -> &mut Self {
        let Some(element) = self.elements.pop() else {
            self.set_error(JatsEncodeError::ElementStackUnderflow);
            return self;
        };

        let is_empty = element
            .content_start
            .is_none_or(|content_start| content_start == self.content.len());
        if is_empty {
            self.content.truncate(element.start);
        } else {
            self.content.push_str("</");
            self.content.push_str(&element.name);
            self.content.push('>');
        }
        self
    }

    /// Add a single encoding loss.
    pub fn add_loss(&mut self, label: impl AsRef<str>) -> &mut Self {
        self.losses.add(label);
        self
    }

    /// Merge encoding losses into the context.
    pub fn merge_losses(&mut self, losses: Losses) -> &mut Self {
        self.losses.merge(losses);
        self
    }

    /// Register and return a stable, unique XML identifier for a reference.
    pub fn register_reference_id(&mut self, preferred: Option<&str>, fallback: &str) -> String {
        let preferred = preferred.map(str::trim).filter(|id| !id.is_empty());
        let preferred_is_usable =
            preferred.is_some_and(|id| valid_xml_id(id) && !self.used_reference_ids.contains(id));
        let candidate = preferred
            .filter(|id| valid_xml_id(id) && !self.used_reference_ids.contains(*id))
            .unwrap_or(fallback);

        if preferred.is_some() && !preferred_is_usable {
            self.losses.add("Reference.id");
        }

        let mut resolved = candidate.to_string();
        let mut suffix = 2;
        while self.used_reference_ids.contains(&resolved) {
            resolved = format!("{fallback}-{suffix}");
            suffix += 1;
        }
        self.used_reference_ids.insert(resolved.clone());

        if let Some(source) = preferred {
            self.reference_ids
                .entry(source.to_string())
                .or_insert_with(|| resolved.clone());
        }

        resolved
    }

    /// Resolve a citation target to the identifier used for its reference.
    pub fn resolve_reference_id<'a>(&'a self, target: &'a str) -> &'a str {
        self.reference_ids
            .get(target)
            .map(String::as_str)
            .unwrap_or(target)
    }

    /// Complete encoding and return its content and losses.
    pub fn finish(mut self) -> Result<(String, Losses), JatsEncodeError> {
        if self.error.is_none() && !self.elements.is_empty() {
            self.error = Some(JatsEncodeError::UnclosedElements(
                self.elements
                    .iter()
                    .map(|element| element.name.clone())
                    .collect(),
            ));
        }

        match self.error {
            Some(error) => Err(error),
            None => Ok((self.content, self.losses)),
        }
    }

    fn close_start_tag(&mut self) {
        if let Some(element) = self.elements.last_mut()
            && element.content_start.is_none()
        {
            self.content.push('>');
            element.content_start = Some(self.content.len());
        }
    }

    fn set_error(&mut self, error: JatsEncodeError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }
}

/// An error caused by invalid use of the JATS encoding context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JatsEncodeError {
    AttributeOutsideStartTag(String),
    ElementStackUnderflow,
    UnclosedElements(Vec<String>),
}

impl fmt::Display for JatsEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttributeOutsideStartTag(name) => {
                write!(
                    formatter,
                    "attribute `{name}` added outside an open start tag"
                )
            }
            Self::ElementStackUnderflow => {
                formatter.write_str("attempted to exit an element when none was open")
            }
            Self::UnclosedElements(names) => {
                write!(formatter, "unclosed JATS elements: {}", names.join(", "))
            }
        }
    }
}

impl std::error::Error for JatsEncodeError {}

fn valid_xml_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|char| char == '_' || char.is_alphabetic())
        && chars.all(|char| {
            char == '_' || char == '-' || char == '.' || char == ':' || char.is_alphanumeric()
        })
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl JatsCodec for $type {
            fn to_jats(&self, context: &mut JatsEncodeContext) {
                context
                    .push_text(self.to_string())
                    .add_loss(concat!($name, "@"));
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl JatsCodec for String {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context.push_text(self);
    }
}

impl<T> JatsCodec for Box<T>
where
    T: JatsCodec,
{
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        self.as_ref().to_jats(context)
    }
}

impl<T> JatsCodec for Option<T>
where
    T: JatsCodec,
{
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        if let Some(value) = self {
            value.to_jats(context);
        }
    }
}

impl<T> JatsCodec for Vec<T>
where
    T: JatsCodec,
{
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        for item in self {
            item.to_jats(context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_nested_xml_and_escapes_values() -> Result<(), JatsEncodeError> {
        let mut context = JatsEncodeContext::default();
        context
            .enter_elem("article")
            .push_attr("title", "A & B")
            .enter_elem("p")
            .push_text("x < y")
            .exit_elem()
            .exit_elem();

        let (xml, losses) = context.finish()?;
        assert_eq!(
            xml,
            "<article title=\"A &amp; B\"><p>x &lt; y</p></article>"
        );
        assert!(losses.is_empty());
        Ok(())
    }

    #[test]
    fn omits_empty_nested_elements() -> Result<(), JatsEncodeError> {
        let mut context = JatsEncodeContext::default();
        context
            .enter_elem("front")
            .enter_elem("abstract")
            .exit_elem_omit_empty()
            .exit_elem_omit_empty();

        let (xml, ..) = context.finish()?;
        assert!(xml.is_empty());
        Ok(())
    }

    #[test]
    fn reports_invalid_writer_order() {
        let mut context = JatsEncodeContext::default();
        context
            .enter_elem("p")
            .push_text("text")
            .push_attr("id", "p1");
        assert!(matches!(
            context.finish(),
            Err(JatsEncodeError::AttributeOutsideStartTag(name)) if name == "id"
        ));
    }

    #[test]
    fn writes_trusted_xml_and_reports_unclosed_elements() -> Result<(), JatsEncodeError> {
        let mut context = JatsEncodeContext::default();
        context
            .enter_elem("mml:math")
            .push_xml("<mml:mi>x</mml:mi>")
            .exit_elem();
        let (xml, ..) = context.finish()?;
        assert_eq!(xml, "<mml:math><mml:mi>x</mml:mi></mml:math>");

        let mut context = JatsEncodeContext::default();
        context.enter_elem("article");
        assert!(matches!(
            context.finish(),
            Err(JatsEncodeError::UnclosedElements(names)) if names == ["article"]
        ));

        let mut context = JatsEncodeContext::default();
        context.exit_elem();
        assert!(matches!(
            context.finish(),
            Err(JatsEncodeError::ElementStackUnderflow)
        ));
        Ok(())
    }

    #[test]
    fn registers_unique_reference_ids() -> Result<(), JatsEncodeError> {
        let mut context = JatsEncodeContext::default();
        assert_eq!(
            context.register_reference_id(Some("smith-2020"), "ref1"),
            "smith-2020"
        );
        assert_eq!(
            context.register_reference_id(Some("smith-2020"), "ref2"),
            "ref2"
        );
        assert_eq!(context.register_reference_id(Some("123"), "ref3"), "ref3");
        assert_eq!(context.resolve_reference_id("smith-2020"), "smith-2020");
        let (.., losses) = context.finish()?;
        assert!(!losses.is_empty());
        Ok(())
    }
}
