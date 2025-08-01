//! Utility function for conversion between formats
//!
//! This crate provides one-way conversion between pairs of formats. It is used
//! internally only. It does not use the Stencila Schema as an intermediate
//! representation of documents as do the `Codec` implementations and leans
//! heavily on external tools.

pub mod html_to_png;
pub use html_to_png::{html_to_png_data_uri, html_to_png_file};

mod latex_to_pdf;
pub use latex_to_pdf::*;

mod pdf_to_md;
pub use pdf_to_md::*;
