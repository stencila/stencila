use codec::{format::Format, Losses};

use crate::lexical::TextFormat;

/// The context for decoding from Lexical JSON
#[derive(Default)]
pub(super) struct LexicalDecodeContext {
    pub losses: Losses,
}


/// The context for encoding to Lexical JSON
#[derive(Default)]
pub(super) struct LexicalEncodeContext {
    pub format: Format,
    pub text_format: TextFormat,
    pub losses: Losses,
}
