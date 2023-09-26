//! Provides the `JatsCodec` trait for generating JATS XML for Stencila Schema nodes

use quick_xml::escape::escape;

pub use codec_jats_derive::JatsCodec;
use codec_losses::Losses;

pub mod encode;

pub trait JatsCodec {
    /// Encode a Stencila Schema node to JATS XML
    fn to_jats(&self) -> (String, Losses) {
        let (name, attrs, content, losses) = self.to_jats_parts();
        (encode::elem(name, attrs, content), losses)
    }

    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses);
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl JatsCodec for $type {
            fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
                (
                    String::new(),
                    Vec::new(),
                    self.to_string(),
                    Losses::one(concat!($name, "@")),
                )
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl JatsCodec for String {
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        (
            String::new(),
            Vec::new(),
            escape(self).to_string(),
            Losses::none(),
        )
    }
}

impl<T> JatsCodec for Box<T>
where
    T: JatsCodec,
{
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        self.as_ref().to_jats_parts()
    }
}

impl<T> JatsCodec for Option<T>
where
    T: JatsCodec,
{
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        match self {
            Some(value) => value.to_jats_parts(),
            None => (String::new(), Vec::new(), String::new(), Losses::none()),
        }
    }
}

impl<T> JatsCodec for Vec<T>
where
    T: JatsCodec,
{
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        let mut jats = String::new();
        let mut losses = Losses::none();

        for item in self.iter() {
            let (item_jats, mut item_losses) = item.to_jats();
            jats.push_str(&item_jats);
            losses.add_all(&mut item_losses);
        }

        (String::new(), Vec::new(), jats, losses)
    }
}
