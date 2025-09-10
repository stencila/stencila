use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Admonition, AppendixBreak, Article, Block, ForBlock, Heading, IncludeBlock, Node, Section,
    SectionType, StyledBlock, VisitorMut, WalkControl,
};

use crate::{should_remove_block, StructuringOperation::*, StructuringOptions};

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_schema::shortcuts::{h1, h2, h3, p, t};

    use super::*;

    #[test]
    fn test_single_section_creation() {
        let mut blocks = vec![
            h1([t("Introduction")]),
            p([t("This is the introduction content.")]),
            p([t("More introduction content.")]),
        ];

        let mut sectioner = ThirdWalk::default();
        sectioner.visit_blocks(&mut blocks);

        assert_eq!(blocks.len(), 1);

        let Block::Section(section) = &blocks[0] else {
            panic!("Expected a Section block");
        };

        assert_eq!(section.section_type, Some(SectionType::Introduction));
        assert_eq!(section.content.len(), 3); // heading + 2 paragraphs

        // First block should be the heading
        assert!(matches!(section.content[0], Block::Heading(_)));

        // Rest should be paragraphs
        assert!(matches!(section.content[1], Block::Paragraph(_)));
        assert!(matches!(section.content[2], Block::Paragraph(_)));
    }

    #[test]
    fn test_nested_sections() {
        let mut blocks = vec![
            h1([t("Methods")]),
            p([t("Methods overview.")]),
            h2([t("Data Collection")]),
            p([t("Data collection details.")]),
            h2([t("Data analysis")]),
            p([t("Data analysis details.")]),
            h1([t("Results")]),
            p([t("Results content.")]),
        ];

        let mut sectioner = ThirdWalk::default();
        sectioner.visit_blocks(&mut blocks);

        assert_eq!(blocks.len(), 2); // Two top-level sections

        // First section: Methods
        let Block::Section(methods_section) = &blocks[0] else {
            panic!("Expected a Section block");
        };
        assert_eq!(methods_section.section_type, Some(SectionType::Methods));
        assert_eq!(methods_section.content.len(), 4); // heading + paragraph + 2 nested sections

        // Check nested sections
        let Block::Section(data_section) = &methods_section.content[2] else {
            panic!("Expected nested Section block");
        };
        assert_eq!(data_section.content.len(), 2); // heading + paragraph

        let Block::Section(analysis_section) = &methods_section.content[3] else {
            panic!("Expected nested Section block");
        };
        assert_eq!(analysis_section.content.len(), 2); // heading + paragraph

        // Second section: Results
        let Block::Section(results_section) = &blocks[1] else {
            panic!("Expected a Section block");
        };
        assert_eq!(results_section.section_type, Some(SectionType::Results));
        assert_eq!(results_section.content.len(), 2); // heading + paragraph
    }

    #[test]
    fn test_mixed_content() {
        let mut blocks = vec![
            p([t("Some content before headings.")]),
            h1([t("Introduction")]),
            p([t("Introduction content.")]),
            p([t("Content between sections.")]),
            h1([t("Methods")]),
            p([t("Methods content.")]),
        ];

        let mut sectioner = ThirdWalk::default();
        sectioner.visit_blocks(&mut blocks);

        assert_eq!(blocks.len(), 3); // paragraph + section + section

        // First block should be the standalone paragraph
        assert!(matches!(blocks[0], Block::Paragraph(_)));

        // Second block should be Introduction section
        let Block::Section(intro_section) = &blocks[1] else {
            panic!("Expected a Section block");
        };
        assert_eq!(intro_section.section_type, Some(SectionType::Introduction));
        // Should contain heading + intro content + content between sections
        assert_eq!(intro_section.content.len(), 3);

        // Third block should be Methods section
        let Block::Section(methods_section) = &blocks[2] else {
            panic!("Expected a Section block");
        };
        assert_eq!(methods_section.section_type, Some(SectionType::Methods));
        assert_eq!(methods_section.content.len(), 2); // heading + content
    }

    #[test]
    fn test_unknown_section_type() {
        let mut blocks = vec![
            h1([t("Custom Section Name")]),
            p([t("Custom section content.")]),
        ];

        let mut sectioner = ThirdWalk::default();
        sectioner.visit_blocks(&mut blocks);

        let Block::Section(section) = &blocks[0] else {
            panic!("Expected a Section block");
        };

        // Unknown section type should be None
        assert_eq!(section.section_type, None);
    }

    #[test]
    fn test_deep_nesting() {
        let mut blocks = vec![
            h1([t("Methods")]),
            p([t("Methods overview.")]),
            h2([t("Data Collection")]),
            p([t("Data collection overview.")]),
            h3([t("Survey Design")]),
            p([t("Survey design details.")]),
            h3([t("Sampling Method")]),
            p([t("Sampling method details.")]),
            h2([t("Analysis")]),
            p([t("Analysis details.")]),
        ];

        let mut sectioner = ThirdWalk::default();
        sectioner.visit_blocks(&mut blocks);

        assert_eq!(blocks.len(), 1); // One top-level section

        let Block::Section(methods_section) = &blocks[0] else {
            panic!("Expected a Section block");
        };
        assert_eq!(methods_section.section_type, Some(SectionType::Methods));
        assert_eq!(methods_section.content.len(), 4); // heading + paragraph + 2 nested sections

        // Check first nested section (Data Collection)
        let Block::Section(data_section) = &methods_section.content[2] else {
            panic!("Expected nested Section block");
        };
        assert_eq!(data_section.content.len(), 4); // heading + paragraph + 2 nested sections

        // Check deeply nested sections
        let Block::Section(survey_section) = &data_section.content[2] else {
            panic!("Expected deeply nested Section block");
        };
        assert_eq!(survey_section.content.len(), 2); // heading + paragraph

        let Block::Section(sampling_section) = &data_section.content[3] else {
            panic!("Expected deeply nested Section block");
        };
        assert_eq!(sampling_section.content.len(), 2); // heading + paragraph
    }
}
