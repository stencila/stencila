//! Provides the `MarkdownCodec` trait for generating Markdown for Stencila Schema nodes

use codec_losses::Losses;

pub use codec_markdown_derive::MarkdownCodec;

pub trait MarkdownCodec {
    /// Encode a Stencila Schema node to Markdown
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses);
}

#[derive(Default, Clone)]
pub struct MarkdownEncodeContext {
    /// The nesting depth for any node type using fenced divs
    pub depth: usize,

    /// The footnotes for the context
    pub footnotes: Vec<String>,
}

impl MarkdownEncodeContext {
    pub fn down(&mut self) {
        self.depth += 1;
    }

    pub fn up(&mut self) {
        self.depth -= 1;
    }
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl MarkdownCodec for $type {
            fn to_markdown(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
                (self.to_string(), Losses::one(concat!($name, "@")))
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl MarkdownCodec for String {
    fn to_markdown(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}

impl<T> MarkdownCodec for Box<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        self.as_ref().to_markdown(context)
    }
}

impl<T> MarkdownCodec for Option<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        match self {
            Some(value) => value.to_markdown(context),
            None => (String::new(), Losses::none()),
        }
    }
}

impl<T> MarkdownCodec for Vec<T>
where
    T: MarkdownCodec,
{
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for item in self.iter() {
            let (item_text, item_losses) = item.to_markdown(context);
            text.push_str(&item_text);
            losses.merge(item_losses);
        }

        (text, losses)
    }
}
