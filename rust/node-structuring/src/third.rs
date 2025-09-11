use stencila_codec::{StructuringOperation::*, StructuringOptions};
use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Admonition, AppendixBreak, Article, Block, ForBlock, Heading, IfBlockClause, IncludeBlock,
    ListItem, Node, Section, SectionType, StyledBlock, TableCell, VisitorMut, WalkControl,
};

use crate::should_remove_block;

/// Third structuring walk
///
/// Walks over a node and performs a third round of structuring focussed on
/// block content (e.g creating [`Section`]s from [`Heading`]s).
#[derive(Debug, Default)]
pub(super) struct ThirdWalk {
    /// The structuring options
    options: StructuringOptions,

    /// Whether the first appendix section has been encountered
    first_appendix_seen: bool,
}

impl VisitorMut for ThirdWalk {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::Article(Article { content, .. }) = node {
            self.visit_blocks(content);
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            // Process nested block content
            Block::Admonition(Admonition { content, .. })
            | Block::IncludeBlock(IncludeBlock {
                content: Some(content),
                ..
            })
            | Block::StyledBlock(StyledBlock { content, .. }) => self.visit_blocks(content),
            Block::ForBlock(ForBlock {
                content,
                iterations,
                ..
            }) => {
                self.visit_blocks(content);
                if let Some(iterations) = iterations {
                    self.visit_blocks(iterations);
                }
                WalkControl::Continue
            }

            // Skip Section blocks to avoid infinite recursion
            Block::Section(_) => WalkControl::Continue,

            _ => WalkControl::Continue,
        }
    }

    fn visit_list_item(&mut self, list_item: &mut ListItem) -> WalkControl {
        self.visit_blocks(&mut list_item.content);
        WalkControl::Continue
    }

    fn visit_table_cell(&mut self, table_cell: &mut TableCell) -> WalkControl {
        self.visit_blocks(&mut table_cell.content);
        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &mut IfBlockClause) -> WalkControl {
        self.visit_blocks(&mut clause.content);
        WalkControl::Continue
    }

    fn visit_walkthrough_step(
        &mut self,
        step: &mut stencila_schema::WalkthroughStep,
    ) -> WalkControl {
        self.visit_blocks(&mut step.content);
        WalkControl::Continue
    }
}

impl ThirdWalk {
    pub fn new(options: StructuringOptions) -> Self {
        Self {
            options,
            ..Default::default()
        }
    }

    /// Visit a vector of blocks and restructure them into nested sections
    ///
    /// This method transforms a flat list of blocks containing headings into
    /// a hierarchical structure of sections based on heading levels.
    fn visit_blocks(&mut self, blocks: &mut Vec<Block>) -> WalkControl {
        let mut blocks_new = Vec::new();
        let mut index = 0;

        while index < blocks.len() {
            let block = &blocks[index];

            if should_remove_block(block) {
                index += 1;
            } else if self.options.should_perform(HeadingsToSections)
                && let Block::Heading(heading) = block
            {
                let (section, consumed) = create_section_from_heading(heading, &blocks[index..]);

                // Check if this is the first appendix section and insert AppendixBreak if so
                if matches!(section.section_type, Some(SectionType::Appendix))
                    && !self.first_appendix_seen
                {
                    self.first_appendix_seen = true;
                    blocks_new.push(Block::AppendixBreak(AppendixBreak::new()));
                }

                blocks_new.push(Block::Section(section));
                index += consumed;
            } else {
                blocks_new.push(block.clone());
                index += 1;
            }
        }

        // Replace the original blocks with the restructured ones
        *blocks = blocks_new;

        WalkControl::Continue
    }
}

/// Create a section from a heading and collect its content
///
/// Returns a tuple of (Section, number_of_blocks_consumed)
fn create_section_from_heading(heading: &Heading, remaining_blocks: &[Block]) -> (Section, usize) {
    let heading_level = heading.level;
    let heading_text = to_text(&heading.content);

    // Determine section type from heading text
    let section_type = SectionType::from_text(&heading_text).ok();

    // Start with the heading as the first block
    let mut section_content = vec![remaining_blocks[0].clone()];
    let mut consumed = 1;

    // Collect blocks until we find another heading of the same or higher level
    let mut index = 1;
    while index < remaining_blocks.len() {
        let block = &remaining_blocks[index];

        match block {
            Block::Heading(other_heading) => {
                if other_heading.level <= heading_level {
                    // Found a heading at same or higher level - stop collecting
                    break;
                } else {
                    // Found a lower-level heading - create a nested section
                    let (nested_section, nested_consumed) =
                        create_section_from_heading(other_heading, &remaining_blocks[index..]);
                    section_content.push(Block::Section(nested_section));
                    index += nested_consumed;
                    consumed += nested_consumed;
                    continue;
                }
            }
            _ => {
                // Regular block - add to section content
                section_content.push(block.clone());
                index += 1;
                consumed += 1;
            }
        }
    }

    let mut section = Section::new(section_content);
    section.section_type = section_type;

    (section, consumed)
}
