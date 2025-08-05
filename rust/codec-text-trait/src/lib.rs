//! Trait for encoding to plain text

pub use codec_text_derive::TextCodec;

/// Encode a node that implements `TextCodec` to plain text
pub fn to_text<T>(node: &T) -> String
where
    T: TextCodec,
{
    let mut text = node.to_text();

    if text.ends_with("\n\n") {
        text.pop();
    }

    text
}

pub trait TextCodec {
    /// Encode a Stencila Schema node as a UTF8 string of text
    ///
    /// Only encodes the primary "content" of a node. For example for
    /// `CodeChunk` nodes, only the `code` property is encoded and
    /// `programminglanguage`, `outputs`, and all other properties are ignored.
    ///
    /// Given that this encoding is inherently lossy, for performance reasons,
    /// this codec does not record losses.
    fn to_text(&self) -> String;
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl TextCodec for $type {
            fn to_text(&self) -> String {
                self.to_string()
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl TextCodec for String {
    fn to_text(&self) -> String {
        self.to_string()
    }
}

impl<T> TextCodec for Box<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> String {
        self.as_ref().to_text()
    }
}

impl<T> TextCodec for Option<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> String {
        match self {
            Some(value) => value.to_text(),
            None => String::new(),
        }
    }
}

impl<T> TextCodec for Vec<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> String {
        let mut text = String::new();

        for item in self {
            let item_text = item.to_text();
            text.push_str(&item_text);
        }

        text
    }
}
