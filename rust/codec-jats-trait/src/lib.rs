//! Provides the `JatsCodec` trait for generating JATS XML for Stencila Schema nodes

pub use codec_jats_derive::JatsCodec;
use codec_losses::{Loss, LossDirection, Losses};

pub trait JatsCodec {
    /// Encode a Stencila Schema node to JATS XML
    fn to_jats(&self) -> (String, Losses) {
        (String::new(), Losses::none())
    }
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl JatsCodec for $type {
            fn to_jats(&self) -> (String, Losses) {
                (
                    self.to_string(),
                    Losses::new([Loss::of_type(LossDirection::Encode, $name)]),
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
    fn to_jats(&self) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}

impl<T> JatsCodec for Box<T>
where
    T: JatsCodec,
{
    fn to_jats(&self) -> (String, Losses) {
        self.as_ref().to_jats()
    }
}

impl<T> JatsCodec for Option<T>
where
    T: JatsCodec,
{
    fn to_jats(&self) -> (String, Losses) {
        match self {
            Some(value) => value.to_jats(),
            None => (String::new(), Losses::none()),
        }
    }
}

impl<T> JatsCodec for Vec<T>
where
    T: JatsCodec,
{
    fn to_jats(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for item in self.iter() {
            let (item_text, mut item_losses) = item.to_jats();
            text.push_str(&item_text);
            losses.add_all(&mut item_losses);
        }

        (text, losses)
    }
}
