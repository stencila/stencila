//! Implements `FromText` and `ToText` traits for encoding and decoding nodes from/to UTF8 strings

use codec::common::eyre::Result;

mod text;

pub trait FromText: Sized {
    /// Decode a node from a UTF8 string of text
    fn from_text(text: &str) -> Result<Self>;
}

pub trait ToText {
    /// Encode a node as a UTF8 string of text
    fn to_text(&self) -> Result<String>;
}
