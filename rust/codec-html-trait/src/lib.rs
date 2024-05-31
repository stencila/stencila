//! Provides the `HtmlCodec` trait for generating  HTML for Stencila Schema nodes
//!
//! Note that this trait can not be in the `codec-html` crate (like, for example, the
//! `Json` trait is in the `codec-json` crate) because `HtmlCodec` is required by
//! the `schema` crate, which is itself a dependency of the `codec` crate (i.e. it would
//! create a circular dependency).

mod boolean;
mod r#box;
mod integer;
mod number;
mod option;
mod string;
mod unsigned_integer;
mod vec;

pub use codec_html_derive::HtmlCodec;

pub mod encode;

#[derive(Default)]
pub struct HtmlEncodeContext {}

pub trait HtmlCodec {
    /// Encode a Stencila Schema node to HTML
    fn to_html(&self, context: &mut HtmlEncodeContext) -> String {
        let parts = self.to_html_parts(context);
        encode::elem(parts.0, &parts.1, &parts.2)
    }

    /// Encode a Stencila Schema node as the parts of a HTML element
    ///
    /// Implementations should return the element name, a vector of
    /// filly formed attribute strings, and a vector of child elements.
    fn to_html_parts(&self, context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>);

    /// Encode a Stencila Schema node as an HTML attribute **value**
    ///
    /// Implementations should return the node represented as a JSON
    /// string (including quotes around strings) so that the `Vec`
    /// implementation of this method can construct valid JSON. The
    /// `encode::attr` function will deal with superfluous quotes.
    fn to_html_attr(&self, context: &mut HtmlEncodeContext) -> String;
}
