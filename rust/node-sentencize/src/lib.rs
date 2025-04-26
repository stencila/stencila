use schema::{Block, Inline, Paragraph, Sentence, Text, VisitorMut, WalkControl, WalkNode};

/// Transform inlines within [`Paragraph`]s into [`Sentence`]s
pub fn sentencize<T: WalkNode>(node: &mut T) {
    let mut walker = Walker;
    node.walk_mut(&mut walker);
}

struct Walker;

impl VisitorMut for Walker {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::Paragraph(Paragraph { content, .. }) = block {
            transform(content)
        }

        WalkControl::Continue
    }
}

/// Transform a vector of inlines into a vector of sentences
fn transform(inlines: &mut Vec<Inline>) {
    let mut sentences = Vec::with_capacity(inlines.len());

    let mut sentence = Vec::new();
    for mut inline in inlines.drain(..) {
        if let Inline::Text(Text { value, .. }) = &mut inline {
            for sentence_text in value.split_inclusive(". ") {
                sentence.push(Inline::Text(Text::new(sentence_text.into())));

                if sentence_text.ends_with(". ") {
                    sentences.push(Inline::Sentence(Sentence::new(
                        sentence.drain(..).collect(),
                    )));
                }
            }
        } else {
            sentence.push(inline);
        }
    }

    if !sentence.is_empty() {
        sentences.push(Inline::Sentence(Sentence::new(sentence)));
    }

    inlines.append(&mut sentences)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use schema::shortcuts::{ci, ct, p, sen, t};

    use super::*;

    macro_rules! sents {
        ($node:expr) => {{
            let mut n = $node;
            sentencize(&mut n);
            n
        }};
    }

    #[test]
    fn basic() {
        assert_eq!(sents!(p([t("one")])), p([sen([t("one")])]));

        assert_eq!(
            sents!(p([t("one. two")])),
            p([sen([t("one. ")]), sen([t("two")])])
        );

        assert_eq!(
            sents!(p([t("one. two. three.")])),
            p([sen([t("one. ")]), sen([t("two. ")]), sen([t("three.")])])
        );

        assert_eq!(
            sents!(p([t("one "), ci("code"), t(".")])),
            p([sen([t("one "), ci("code"), t(".")])])
        );

        assert_eq!(
            sents!(p([t("one "), ct("target"), t(".")])),
            p([sen([t("one "), ct("target"), t(".")])])
        );
    }
}
