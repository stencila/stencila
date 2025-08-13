use codec_text_trait::to_text;
use schema::{Block, List, Node, Reference, VisitorMut, WalkControl, WalkNode};

/// Add structure to a document
pub fn structuring<T: WalkNode>(node: &mut T) {
    let mut walker = Walker::default();
    node.walk_mut(&mut walker);
}

#[derive(Debug, Default)]
struct Walker {
    /// Whether currently in the References section
    in_references: bool,

    /// References collected from walking document
    references: Option<Vec<Reference>>,
}

impl VisitorMut for Walker {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::Article(article) => {
                self.walk(article);

                // If any references were collected then
                if let Some(references) = self.references.take() {
                    article.references = Some(references);
                }
            }

            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Heading(heading) => {
                // Detect if entering references section
                let text = to_text(&heading.content).to_lowercase();
                if matches!(text.trim(), "references" | "bibliography") {
                    self.in_references = true;
                } else {
                    if heading.level <= 3 {
                        self.in_references = false;
                    }
                }
            }

            Block::List(list) => {
                if self.in_references {
                    self.list_to_references(list)
                }
            }

            _ => {}
        }

        WalkControl::Continue
    }
}

impl Walker {
    /// Transform a [`List`] to a set of [`Reference`]s to assign to the root node
    fn list_to_references(&mut self, list: &List) {
        let mut references = Vec::new();
        for (index, item) in list.items.iter().enumerate() {
            let text = to_text(item);
            if let Some(reference) = codec_biblio::decode::text(&text)
                .ok()
                .and_then(|mut refs| refs.pop())
            {
                references.push(Reference {
                    id: Some((index + 1).to_string()),
                    ..reference
                });
            };
        }
        self.references = (!references.is_empty()).then_some(references);
    }
}
