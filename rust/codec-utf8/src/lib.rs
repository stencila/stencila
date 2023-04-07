//! Implements `FromUtf8` and `ToUtf8` traits for encoding and decoding nodes from/to UTF8 strings
 
use common::eyre::Result;

mod text;

pub trait FromUtf8: Sized {
    /// Decode a node from a UTF8 string
    fn from_utf8(utf8: &str) -> Result<Self>;
}

pub trait ToUtf8 {
    /// Encode a node as a UTF8 string
    fn to_utf8(&self) -> Result<String>;
}
