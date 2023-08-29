use std::ops::Deref;

use codec_losses::{Loss, LossDirection, Losses};

pub trait ToText {
    /// Encode a node as a UTF8 string of text
    fn to_text(&self) -> (String, Losses);
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl ToText for $type {
            fn to_text(&self) -> (String, Losses) {
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

impl ToText for String {
    fn to_text(&self) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}

impl<T> ToText for Box<T>
where
    T: ToText,
{
    fn to_text(&self) -> (String, Losses) {
        self.deref().to_text()
    }
}

impl<T> ToText for Option<T>
where
    T: ToText,
{
    fn to_text(&self) -> (String, Losses) {
        match self {
            Some(value) => value.to_text(),
            None => (String::new(), Losses::none()),
        }
    }
}

impl<T> ToText for Vec<T>
where
    T: ToText,
{
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, mut item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.add_all(&mut item_losses);
        }

        (text, losses)
    }
}
