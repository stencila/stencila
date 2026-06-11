use stencila_codec_info::lost_options;

use super::research_block::{
    push_extra_options, push_qmd_attr, push_qmd_extra_attrs, push_qmd_relation_attrs,
    push_qmd_title, push_relation_options,
};
use crate::{Claim, prelude::*};

impl MarkdownCodec for Claim {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, authors, provenance));

        let claim_type = self
            .claim_type
            .map(|claim_type| claim_type.to_string().to_lowercase());

        if matches!(context.format, Format::Myst) {
            let directive = claim_type
                .as_ref()
                .map(|claim_type| ["prf:", claim_type].concat())
                .unwrap_or_else(|| "claim".to_string());

            context
                .myst_directive(
                    ':',
                    &directive,
                    |context| {
                        if let Some(title) = &self.options.title {
                            context
                                .push_str(" ")
                                .push_prop_fn(NodeProperty::Title, |context| {
                                    title.to_markdown(context)
                                });
                        }
                    },
                    |context| {
                        if let Some(id) = &self.id {
                            context.myst_directive_option(NodeProperty::Id, None, id);
                        }

                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }

                        push_relation_options(context, &self.relations);
                        push_extra_options(context, &self.options.extra);
                    },
                    |context| {
                        context.push_prop_fn(NodeProperty::Content, |context| {
                            self.content.to_markdown(context)
                        });
                    },
                )
                .exit_node()
                .newline();
        } else if matches!(context.format, Format::Qmd) {
            context.push_colons().push_str(" {.");
            if let Some(claim_type) = claim_type.as_deref() {
                context.push_prop_str(NodeProperty::ClaimType, claim_type);
            } else {
                context.push_str("claim");
            }

            if let Some(id) = &self.id {
                context.push_str(" #").push_prop_str(NodeProperty::Id, id);
            }

            if let Some(label) = &self.label {
                push_qmd_attr(context, NodeProperty::Label, "label", label);
            }

            push_qmd_relation_attrs(context, &self.relations);
            push_qmd_extra_attrs(context, &self.options.extra);

            context.push_str("}\n\n");
            push_qmd_title(context, &self.options.title);

            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .push_colons()
                .newline()
                .exit_node()
                .newline();
        } else {
            context.push_colons().push_str(" ");
            if let Some(claim_type) = claim_type {
                context.push_prop_str(NodeProperty::ClaimType, &claim_type);
            } else {
                context.push_str("claim");
            }

            if let Some(label) = &self.label {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::Label, label);
            }

            if let Some(id) = &self.id {
                context.push_str(" #").push_prop_str(NodeProperty::Id, id);
            }

            context.newline();

            push_relation_options(context, &self.relations);
            push_extra_options(context, &self.options.extra);

            context
                .newline()
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .push_colons()
                .newline()
                .exit_node()
                .newline();
        }
    }
}

#[cfg(test)]
mod tests {
    use stencila_codec_markdown_trait::to_markdown_flavor;

    use crate::{
        ClaimType, ResearchObjectRelation, ResearchObjectRelationKind,
        shortcuts::{p, t},
    };

    use super::*;

    fn supported_by_e1() -> Vec<ResearchObjectRelation> {
        vec![ResearchObjectRelation::new(
            ResearchObjectRelationKind::SupportedBy,
            "#e1".to_string(),
        )]
    }

    #[test]
    fn encode_untyped_claim_to_markdown() {
        let claim = Claim::new(vec![p([t("Claim text.")])]);

        let markdown = to_markdown_flavor(&claim, Format::Smd);

        assert_eq!(markdown, "::: claim\n\nClaim text.\n\n:::");
    }

    #[test]
    fn encode_claim_to_myst() {
        let mut claim = Claim::new(vec![p([t("Claim text.")])]);
        claim.claim_type = Some(ClaimType::Statement);
        claim.id = Some("c1".to_string());
        claim.label = Some("Claim 1".to_string());
        claim.options.title = Some(vec![t("Core claim")]);
        claim.relations = Some(supported_by_e1());

        let markdown = to_markdown_flavor(&claim, Format::Myst);

        assert_eq!(
            markdown,
            ":::{prf:statement} Core claim\n:id: c1\n:label: Claim 1\n:supported-by: #e1\n\nClaim text.\n\n:::"
        );
    }

    #[test]
    fn encode_claim_to_qmd() {
        let mut claim = Claim::new(vec![p([t("Claim text.")])]);
        claim.claim_type = Some(ClaimType::Statement);
        claim.id = Some("c1".to_string());
        claim.label = Some("Claim 1".to_string());
        claim.options.title = Some(vec![t("Core claim")]);
        claim.relations = Some(supported_by_e1());

        let markdown = to_markdown_flavor(&claim, Format::Qmd);

        assert_eq!(
            markdown,
            "::: {.statement #c1 label=\"Claim 1\" supported-by=\"#e1\"}\n\n## Core claim\n\nClaim text.\n\n:::"
        );
    }
}
