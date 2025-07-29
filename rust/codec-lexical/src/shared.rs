use codec::{Losses, format::Format};

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
    pub standalone: bool,
    pub text_format: TextFormat,
    pub losses: Losses,
}
