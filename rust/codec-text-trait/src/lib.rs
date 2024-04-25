//! Trait for encoding to plain text

use codec_info::Losses;

pub use codec_text_derive::TextCodec;

pub trait TextCodec {
    /// Encode a node as a UTF8 string of text
    fn to_text(&self) -> (String, Losses);
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl TextCodec for $type {
            fn to_text(&self) -> (String, Losses) {
                (self.to_string(), Losses::one(concat!($name, "@")))
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl TextCodec for String {
    fn to_text(&self) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}

impl<T> TextCodec for Box<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> (String, Losses) {
        self.as_ref().to_text()
    }
}

impl<T> TextCodec for Option<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> (String, Losses) {
        match self {
            Some(value) => value.to_text(),
            None => (String::new(), Losses::none()),
        }
    }
}

impl<T> TextCodec for Vec<T>
where
    T: TextCodec,
{
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.merge(item_losses);
        }

        (text, losses)
    }
}
