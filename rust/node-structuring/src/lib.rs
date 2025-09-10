use stencila_codec::StructuringOptions;
use stencila_schema::{Block, CodeInline, Inline, RawBlock, VisitorMut, WalkControl, WalkNode};

use crate::{first::FirstWalk, second::SecondWalk, third::ThirdWalk};

mod first;
mod second;
mod third;

#[cfg(test)]
mod tests;

/// Add structure to a document with default options
pub fn structuring<T: WalkNode>(node: &mut T) {
    structuring_with_options(node, StructuringOptions::default())
}

/// Add structure to a document with custom options
pub fn structuring_with_options<T: WalkNode>(node: &mut T, options: StructuringOptions
) {
    let mut first = FirstWalk::new(options.clone());
    first.walk(node);
    first.determine_citation_style(options.citation_style);

    let mut second = SecondWalk::new(options.clone(), first);
    second.walk(node);

    let mut third = ThirdWalk::new(options);
    third.walk(node);
}

const REMOVE_MARKER: &str = "<remove>";

/// Create a block that is marked for removal in a subsequent walk
fn block_to_remove(block: &mut Block) -> WalkControl {
    *block = Block::RawBlock(RawBlock::new(REMOVE_MARKER.into(), "".into()));
    WalkControl::Break
}

/// Test if a block should be removed in the current walk
fn should_remove_block(block: &Block) -> bool {
    if let Block::RawBlock(RawBlock { format, .. }) = block {
        format == REMOVE_MARKER
    } else {
        false
    }
}

/// Create an inline that is marked for removal in a subsequent walk
fn inline_to_remove(inline: &mut Inline) -> WalkControl {
    *inline = Inline::CodeInline(CodeInline {
        programming_language: Some(REMOVE_MARKER.into()),
        ..Default::default()
    });
    WalkControl::Break
}

/// Test if an inline should be removed in the current walk
fn should_remove_inline(inline: &Inline) -> bool {
    if let Inline::CodeInline(CodeInline {
        programming_language: Some(lang),
        ..
    }) = inline
    {
        lang == REMOVE_MARKER
    } else {
        false
    }
}
