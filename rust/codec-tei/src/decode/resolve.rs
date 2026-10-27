//! Links and citations to elements that were not decoded to kept nodes

use std::collections::HashSet;

use stencila_codec::stencila_schema::{Article, Block, Inline, VisitorMut, WalkControl, WalkNode};

use super::{Context, coords};

/// Remove the targets of links and citations that are not the ids of kept
/// nodes
///
/// The target of a reference is checked to be the id of an element when the
/// reference is decoded, but not every element with an id is decoded to a
/// node with it: GROBID's sentences (`<s>`), for example, are unwrapped. A
/// link or citation with none of its targets left is replaced by its content.
/// Each is recorded as lost, as a reference that does not resolve is.
pub(super) fn remove_dangling_refs(article: &mut Article, context: &mut Context) {
    if context.internal_refs.is_empty() {
        return;
    }

    let mut ids = HashSet::new();
    coords::ids(article, &mut ids);

    let mut remover = Remover { ids, context };
    if let Some(title) = &mut article.title {
        remover.resolve(title);
    }
    article.walk_mut(&mut remover);
}

struct Remover<'c, 'a, 'input> {
    /// The ids of the kept nodes
    ids: HashSet<String>,

    context: &'c mut Context<'a, 'input>,
}

/// What a link or citation resolves to
enum Resolved {
    /// All of its targets were kept
    Whole(Inline),
    /// Only some of its targets were kept
    Partial(Inline),
    /// None of its targets were kept, so only its content remains
    Content(Vec<Inline>),
}

impl Remover<'_, '_, '_> {
    /// Resolve the links and citations within inlines
    fn resolve(&mut self, inlines: &mut Vec<Inline>) {
        if !inlines.iter().any(|inline| self.label(inline).is_some()) {
            return;
        }

        let mut resolved: Vec<Inline> = Vec::with_capacity(inlines.len());
        for inline in std::mem::take(inlines) {
            let Some(label) = self.label(&inline) else {
                push_merged(&mut resolved, inline);
                continue;
            };

            match self.resolve_inline(inline) {
                Resolved::Whole(inline) => push_merged(&mut resolved, inline),
                Resolved::Partial(inline) => {
                    self.context
                        .lost_label(format!("{label} (partially unresolved)"));
                    push_merged(&mut resolved, inline);
                }
                Resolved::Content(content) => {
                    self.context.lost_label(format!("{label} (unresolved)"));
                    for inline in content {
                        push_merged(&mut resolved, inline);
                    }
                }
            }
        }
        *inlines = resolved;
    }

    /// The loss label of a link or citation to elements in the document
    fn label(&self, inline: &Inline) -> Option<String> {
        let node_id = match inline {
            Inline::Link(..) | Inline::Citation(..) | Inline::CitationGroup(..) => {
                inline.node_id()?
            }
            _ => return None,
        };
        self.context.internal_refs.get(&node_id).cloned()
    }

    /// Resolve a link or citation to elements in the document
    fn resolve_inline(&self, inline: Inline) -> Resolved {
        match inline {
            Inline::Link(mut link) => {
                let targets = link.target.split_whitespace().collect::<Vec<_>>();
                let kept = targets
                    .iter()
                    .filter(|target| {
                        target
                            .strip_prefix('#')
                            .is_some_and(|id| self.ids.contains(id))
                    })
                    .copied()
                    .collect::<Vec<_>>();
                if kept.len() == targets.len() {
                    Resolved::Whole(Inline::Link(link))
                } else if kept.is_empty() {
                    Resolved::Content(link.content)
                } else {
                    link.target = kept.join(" ");
                    Resolved::Partial(Inline::Link(link))
                }
            }
            Inline::Citation(citation) => {
                if self.ids.contains(&citation.target) {
                    Resolved::Whole(Inline::Citation(citation))
                } else {
                    Resolved::Content(citation.options.content.unwrap_or_default())
                }
            }
            Inline::CitationGroup(mut group) => {
                let count = group.items.len();
                group
                    .items
                    .retain(|citation| self.ids.contains(&citation.target));
                if group.items.len() == count {
                    Resolved::Whole(Inline::CitationGroup(group))
                } else if group.items.is_empty() {
                    Resolved::Content(group.content.unwrap_or_default())
                } else {
                    Resolved::Partial(Inline::CitationGroup(group))
                }
            }
            inline => Resolved::Whole(inline),
        }
    }
}

/// Add an inline to others, merging it with the last if both are text
fn push_merged(inlines: &mut Vec<Inline>, inline: Inline) {
    if let Inline::Text(text) = &inline
        && let Some(Inline::Text(last)) = inlines.last_mut()
    {
        last.value = [last.value.as_str(), text.value.as_str()].concat().into();
    } else {
        inlines.push(inline);
    }
}

impl VisitorMut for Remover<'_, '_, '_> {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Paragraph(paragraph) => self.resolve(&mut paragraph.content),
            Block::Heading(heading) => self.resolve(&mut heading.content),
            _ => {}
        }
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::Emphasis(mark) => self.resolve(&mut mark.content),
            Inline::Strong(mark) => self.resolve(&mut mark.content),
            Inline::Superscript(mark) => self.resolve(&mut mark.content),
            Inline::Subscript(mark) => self.resolve(&mut mark.content),
            Inline::Underline(mark) => self.resolve(&mut mark.content),
            Inline::Strikeout(mark) => self.resolve(&mut mark.content),
            Inline::QuoteInline(quote) => self.resolve(&mut quote.content),
            Inline::Link(link) => self.resolve(&mut link.content),
            _ => {}
        }
        WalkControl::Continue
    }
}
