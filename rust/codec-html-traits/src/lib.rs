//! Provides the `ToHtml` trait for generating  HTML for Stencila Schema nodes
//!
//! Note that this trait can not be in the `codec-html` crate (like, for examples, the
//! `ToJSon` trait is in the `codec-json` crate) because `ToHtml` is required by
//! the `schema` crate, which is itself a dependency of the `codec` crate (i.e. it would
//! create a circular dependency).

pub mod to_html;
pub use to_html::ToHtml;

mod prelude;

mod boolean;
mod r#box;
mod integer;
mod number;
mod option;
mod string;
mod unsigned_integer;
mod vec;
