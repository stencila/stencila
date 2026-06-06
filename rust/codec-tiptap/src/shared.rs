//! Shared support types used by Tiptap conversion modules.
//!
//! Decode and encode contexts are kept distinct so each path can grow its own
//! settings without coupling the two conversion directions.

use stencila_codec::Losses;

use crate::tiptap::MarkAttrs;

/// The context for decoding from Tiptap JSON.
#[derive(Default)]
pub(super) struct TiptapDecodeContext {
    /// Losses recorded while decoding from Tiptap JSON.
    pub losses: Losses,
}

/// The context for encoding to Tiptap JSON.
#[derive(Default)]
pub(super) struct TiptapEncodeContext {
    /// Losses recorded while encoding to Tiptap JSON.
    pub losses: Losses,
}

/// Text mark state applied while encoding nested Stencila inline nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MarkKind {
    /// Bold text.
    Bold,
    /// Inline code.
    Code(MarkAttrs),
    /// Italic text.
    Italic,
    /// Linked text.
    Link(MarkAttrs),
    /// Struck out text.
    Strikeout,
    /// Subscripted text.
    Subscript,
    /// Superscripted text.
    Superscript,
    /// Underlined text.
    Underline,
}
