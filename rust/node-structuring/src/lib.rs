use stencila_codec::StructuringOperation::Heading1ToTitleSingle;
use stencila_codec::StructuringOptions;
use stencila_schema::{
    Block, CodeInline, Heading, Inline, RawBlock, Visitor, VisitorAsync, VisitorMut, WalkControl,
    WalkNode,
};

use crate::{first::FirstWalk, second::SecondWalk, third::ThirdWalk};

mod first;
mod second;
mod third;

/// Counts of heading levels in a document
///
/// Used by the `Heading1ToTitleSingle` operation to determine
/// whether to extract a title from a single H1 heading.
#[derive(Debug, Default, Clone)]
pub struct HeadingCounts {
    /// Number of level 1 headings
    pub h1_count: usize,
    /// Number of other headings (levels 2-6)
    pub other_count: usize,
}

/// Pre-scan visitor to count headings
///
/// Excludes supplements to match the behavior of FirstWalk, which processes
/// supplements separately with their own walker instance.
#[derive(Debug, Default)]
struct HeadingCounter {
    counts: HeadingCounts,
}

impl Visitor for HeadingCounter {
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        match block {
            // Skip supplements - they are processed separately in FirstWalk
            Block::Supplement(_) => WalkControl::Break,
            Block::Heading(Heading { level, .. }) => {
                if *level == 1 {
                    self.counts.h1_count += 1;
                } else {
                    self.counts.other_count += 1;
                }
                WalkControl::Continue
            }
            _ => WalkControl::Continue,
        }
    }
}

/// Add structure to a document
#[tracing::instrument(skip(node))]
pub async fn structuring<T: WalkNode>(
    node: &mut T,
    options: StructuringOptions,
) -> eyre::Result<()> {
    tracing::trace!("Structuring node");

    // Pre-scan to count headings if Heading1ToTitleSingle is enabled
    let heading_counts = if options.should_perform(Heading1ToTitleSingle) {
        let mut counter = HeadingCounter::default();
        counter.walk(node);
        Some(counter.counts)
    } else {
        None
    };

    let mut first = FirstWalk::new(options.clone(), heading_counts);
    first.walk(node).await?;
    first.determine_citation_style(options.citation_style);

    let mut second = SecondWalk::new(options.clone(), first);
    second.walk(node);

    let mut third = ThirdWalk::new(options);
    third.walk(node);

    Ok(())
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
