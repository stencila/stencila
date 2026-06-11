use stencila_codec_info::lost_options;

use crate::{
    Block, Evidence, Inline, Object, Primitive, Protocol, Question, Request,
    ResearchObjectRelation, ResearchObjectRelationKind, prelude::*,
};

pub(crate) fn push_extra_options(context: &mut MarkdownEncodeContext, extra: &Option<Object>) {
    let Some(extra) = extra else {
        return;
    };

    for (key, value) in extra.iter() {
        // Relation-shaped keys are encoded from `relations`, so skip them here
        // to avoid duplicate attributes when a node carries both (e.g. JSON-sourced)
        if ResearchObjectRelationKind::from_authored_key(key).is_some() {
            continue;
        }

        context
            .push_str(":")
            .push_str(key)
            .push_str(": ")
            .push_prop_str(NodeProperty::Extra, &primitive_to_option_value(value))
            .newline();
    }
}

pub(crate) fn push_qmd_extra_attrs(context: &mut MarkdownEncodeContext, extra: &Option<Object>) {
    let Some(extra) = extra else {
        return;
    };

    for (key, value) in extra.iter() {
        // Relation-shaped keys are encoded from `relations`, so skip them here
        // to avoid duplicate attributes when a node carries both (e.g. JSON-sourced)
        if ResearchObjectRelationKind::from_authored_key(key).is_some() {
            continue;
        }

        push_qmd_attr(
            context,
            NodeProperty::Extra,
            key,
            &primitive_to_option_value(value),
        );
    }
}

/// Group relations into authored attributes: one key per relation kind, in
/// first-appearance order, with multiple targets space separated.
///
/// Decoding splits targets on whitespace and commas, so relations interleaved
/// across kinds re-group by kind on a round-trip; targets must not contain
/// whitespace.
fn grouped_relation_attrs(
    relations: &Option<Vec<ResearchObjectRelation>>,
) -> Vec<(&'static str, String)> {
    let mut groups: Vec<(&'static str, String)> = Vec::new();

    for relation in relations.iter().flatten() {
        let key = relation.kind.authored_key();
        if let Some((_, targets)) = groups.iter_mut().find(|(existing, _)| *existing == key) {
            targets.push(' ');
            targets.push_str(&relation.target);
        } else {
            groups.push((key, relation.target.clone()));
        }
    }

    groups
}

pub(crate) fn push_relation_options(
    context: &mut MarkdownEncodeContext,
    relations: &Option<Vec<ResearchObjectRelation>>,
) {
    for (key, targets) in grouped_relation_attrs(relations) {
        context
            .push_str(":")
            .push_str(key)
            .push_str(": ")
            .push_prop_str(NodeProperty::Relations, &targets)
            .newline();
    }
}

pub(crate) fn push_qmd_relation_attrs(
    context: &mut MarkdownEncodeContext,
    relations: &Option<Vec<ResearchObjectRelation>>,
) {
    for (key, targets) in grouped_relation_attrs(relations) {
        push_qmd_attr(context, NodeProperty::Relations, key, &targets);
    }
}

pub(crate) fn push_qmd_attr(
    context: &mut MarkdownEncodeContext,
    prop: NodeProperty,
    key: &str,
    value: &str,
) {
    context
        .push_str(" ")
        .push_str(key)
        .push_str("=\"")
        .push_prop_str(prop, &qmd_attr_value(value))
        .push_str("\"");
}

pub(crate) fn push_qmd_title(context: &mut MarkdownEncodeContext, title: &Option<Vec<Inline>>) {
    if let Some(title) = title {
        context
            .push_str("## ")
            .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context))
            .push_str("\n\n");
    }
}

fn qmd_attr_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn primitive_to_option_value(value: &Primitive) -> String {
    match value {
        Primitive::String(value) => value.to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| value.to_text()),
    }
}

struct ResearchBlockMarkdown<'a> {
    name: &'a str,
    node_type: NodeType,
    node_id: NodeId,
    id: &'a Option<String>,
    label: &'a Option<String>,
    title: &'a Option<Vec<Inline>>,
    content: &'a Vec<Block>,
    relations: &'a Option<Vec<ResearchObjectRelation>>,
    extra: &'a Option<Object>,
}

fn to_markdown(block: ResearchBlockMarkdown<'_>, context: &mut MarkdownEncodeContext) {
    context.enter_node(block.node_type, block.node_id);

    if matches!(context.format, Format::Myst) {
        context
            .myst_directive(
                ':',
                block.name,
                |context| {
                    if let Some(title) = block.title {
                        context
                            .push_str(" ")
                            .push_prop_fn(NodeProperty::Title, |context| {
                                title.to_markdown(context)
                            });
                    }
                },
                |context| {
                    if let Some(id) = block.id {
                        context.myst_directive_option(NodeProperty::Id, None, id);
                    }

                    if let Some(label) = block.label {
                        context.myst_directive_option(NodeProperty::Label, None, label);
                    }

                    push_relation_options(context, block.relations);
                    push_extra_options(context, block.extra);
                },
                |context| {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        block.content.to_markdown(context)
                    });
                },
            )
            .exit_node()
            .newline();
    } else if matches!(context.format, Format::Qmd) {
        context.push_colons().push_str(" {.");
        context.push_str(block.name);

        if let Some(id) = block.id {
            context.push_str(" #").push_prop_str(NodeProperty::Id, id);
        }

        if let Some(label) = block.label {
            push_qmd_attr(context, NodeProperty::Label, "label", label);
        }

        push_qmd_relation_attrs(context, block.relations);
        push_qmd_extra_attrs(context, block.extra);

        context.push_str("}\n\n");
        push_qmd_title(context, block.title);

        context
            .push_prop_fn(NodeProperty::Content, |context| {
                block.content.to_markdown(context)
            })
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    } else {
        context.push_colons().push_str(" ").push_str(block.name);

        if let Some(label) = block.label {
            context
                .push_str(" ")
                .push_prop_str(NodeProperty::Label, label);
        }

        if let Some(id) = block.id {
            context.push_str(" #").push_prop_str(NodeProperty::Id, id);
        }

        context.newline();
        push_relation_options(context, block.relations);
        push_extra_options(context, block.extra);

        context
            .newline()
            .push_prop_fn(NodeProperty::Content, |context| {
                block.content.to_markdown(context)
            })
            .push_colons()
            .newline()
            .exit_node()
            .newline();
    }
}

macro_rules! impl_research_block {
    ($node:ident, $name:literal) => {
        impl MarkdownCodec for $node {
            fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
                context.merge_losses(lost_options!(self, authors, provenance));
                to_markdown(
                    ResearchBlockMarkdown {
                        name: $name,
                        node_type: self.node_type(),
                        node_id: self.node_id(),
                        id: &self.id,
                        label: &self.label,
                        title: &self.options.title,
                        content: &self.content,
                        relations: &self.relations,
                        extra: &self.options.extra,
                    },
                    context,
                );
            }
        }
    };
}

impl_research_block!(Evidence, "evidence");
impl_research_block!(Protocol, "protocol");
impl_research_block!(Question, "question");
impl_research_block!(Request, "request");

#[cfg(test)]
mod tests {
    use stencila_codec_markdown_trait::to_markdown_flavor;

    use crate::shortcuts::{p, t};

    use super::*;

    fn relation(kind: ResearchObjectRelationKind, target: &str) -> ResearchObjectRelation {
        ResearchObjectRelation::new(kind, target.to_string())
    }

    fn question() -> Question {
        let mut node = Question::new(vec![p([t("What follows?")])]);
        node.id = Some("q1".to_string());
        node.label = Some("Question 1".to_string());
        node.options.title = Some(vec![t("Research question")]);
        node.relations = Some(vec![relation(
            ResearchObjectRelationKind::SupportedBy,
            "#e1",
        )]);
        node
    }

    #[test]
    fn encode_question_to_smd() {
        let markdown = to_markdown_flavor(&question(), Format::Smd);

        assert_eq!(
            markdown,
            "::: question Question 1 #q1\n:supported-by: #e1\n\nWhat follows?\n\n:::"
        );
    }

    #[test]
    fn encode_question_to_myst() {
        let markdown = to_markdown_flavor(&question(), Format::Myst);

        assert_eq!(
            markdown,
            ":::{question} Research question\n:id: q1\n:label: Question 1\n:supported-by: #e1\n\nWhat follows?\n\n:::"
        );
    }

    #[test]
    fn encode_question_to_qmd() {
        let markdown = to_markdown_flavor(&question(), Format::Qmd);

        assert_eq!(
            markdown,
            "::: {.question #q1 label=\"Question 1\" supported-by=\"#e1\"}\n\n## Research question\n\nWhat follows?\n\n:::"
        );
    }

    #[test]
    fn encode_relations_grouped_by_kind() {
        let mut node = Evidence::new(vec![p([t("Evidence text.")])]);
        node.relations = Some(vec![
            relation(ResearchObjectRelationKind::Supports, "#c1"),
            relation(ResearchObjectRelationKind::IsGroundedIn, "#p1"),
            relation(ResearchObjectRelationKind::Supports, "#c2"),
        ]);

        assert_eq!(
            to_markdown_flavor(&node, Format::Qmd),
            "::: {.evidence supports=\"#c1 #c2\" is-grounded-in=\"#p1\"}\n\nEvidence text.\n\n:::"
        );

        assert_eq!(
            to_markdown_flavor(&node, Format::Smd),
            "::: evidence\n:supports: #c1 #c2\n:is-grounded-in: #p1\n\nEvidence text.\n\n:::"
        );
    }

    #[test]
    fn encode_skips_relation_shaped_extra_keys() {
        let mut node = Question::new(vec![p([t("What follows?")])]);
        node.relations = Some(vec![relation(
            ResearchObjectRelationKind::SupportedBy,
            "#e1",
        )]);
        node.options.extra = Some(Object::from([
            ("supported-by", Primitive::String("#stale".to_string())),
            ("source", Primitive::String("survey".to_string())),
        ]));

        assert_eq!(
            to_markdown_flavor(&node, Format::Qmd),
            "::: {.question supported-by=\"#e1\" source=\"survey\"}\n\nWhat follows?\n\n:::"
        );
    }
}
