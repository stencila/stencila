//! Provides the `HtmlCodec` trait for generating  HTML for Stencila Schema nodes
//!
//! Note that this trait can not be in the `codec-html` crate (like, for examples, the
//! `Json` trait is in the `codec-json` crate) because `HtmlCodec` is required by
//! the `schema` crate, which is itself a dependency of the `codec` crate (i.e. it would
//! create a circular dependency).

mod prelude;

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

pub trait HtmlCodec {
    /// Encode a Stencila Schema node to HTML
    fn to_html(&self) -> String;
}
