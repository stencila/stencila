use codec::{Losses, schema::{Block, Inline}};

/// Decode Markdown content to a vector of [`Block`]s
///
/// Intended for decoding a fragment of Markdown (e.g. a Markdown cell in
/// a Jupyter Notebook) and inserting it into a larger document.
pub fn decode_blocks(md: &str) -> (Vec<Block>, Losses) {
    todo!()
}

/// Decode Markdown content to a vector of [`Inline`]s
pub fn decode_inlines(md: &str) -> (Vec<Inline>, Losses) {
    todo!()
}
