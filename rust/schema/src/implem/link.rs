use stencila_codec_info::lost_options;
use stencila_codec_text_trait::to_text;

use crate::{Inline, Link, prelude::*};

impl Link {
    /// Get the ids that an internal link addresses
    ///
    /// A link addresses more than one id when it stands in for a JATS `<xref>`
    /// whose `rid` lists several targets, such as a range of figures.
    fn internal_targets(&self) -> Option<Vec<&str>> {
        let targets = self
            .target
            .split_whitespace()
            .map(|target| target.strip_prefix('#'))
            .collect::<Option<Vec<_>>>()?;

        (!targets.is_empty() && targets.iter().all(|target| !target.is_empty())).then_some(targets)
    }
}

impl JatsCodec for Link {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context.merge_losses(lost_options!(self, label_only, compilation_messages));

        // An internal link is only encodable as an `<xref>` if every id it
        // addresses is on a node that is also being encoded; a `rid` that
        // dangles is worse than no link at all
        if let Some(targets) = self.internal_targets() {
            if self.rel.is_some() {
                context.add_loss("Link.rel");
            }
            if self.jats_ext_link_type.is_some() {
                context.add_loss("Link.jatsExtLinkType");
            }

            let kinds = targets
                .iter()
                .map(|target| context.link_target_ref_type(target).cloned())
                .collect::<Option<Vec<_>>>();

            let Some(kinds) = kinds else {
                // The target was already unresolved in the source, so emit the
                // content without a link and report the dropped target
                context.add_loss("Link.target");
                self.content.to_jats(context);
                return;
            };

            // A cross-reference spanning unlike target kinds uses the general
            // `other` type rather than falsely describing some of its targets.
            let kind = kinds
                .first()
                .filter(|first| kinds.iter().all(|kind| kind == *first))
                .cloned()
                .unwrap_or(JatsRefType::Other);

            // A source `ref-type` is kept when it says something more specific
            // than the kind of the target node, as `table-fn` does of a footnote
            let source_ref_type = self
                .jats_ref_type
                .as_deref()
                .map(JatsRefType::from)
                .filter(|ref_type| ref_type.is_compatible_with(&kind));
            if self.jats_ref_type.is_some() && source_ref_type.is_none() {
                context.add_loss("Link.jatsRefType");
            }
            let ref_type = source_ref_type.as_ref().unwrap_or(&kind);

            context
                .enter_elem("xref")
                .push_attr("ref-type", ref_type.as_str())
                .push_attr("rid", targets.join(" "));
            if let Some(id) = &self.id {
                context.push_attr("id", id);
            }
            if let Some(title) = &self.title {
                context.push_attr("xlink:title", title);
            }
            self.content.to_jats(context);
            context.exit_elem();

            return;
        }

        if self.rel.is_some() {
            // JATS has no counterpart to a HTML link relationship on `<ext-link>`
            context.add_loss("Link.rel");
        }
        if self.jats_ref_type.is_some() {
            context.add_loss("Link.jatsRefType");
        }

        context.enter_elem("ext-link");
        if let Some(id) = &self.id {
            context.push_attr("id", id);
        }
        // Only a target with a scheme is stated to be a URI; JATS uses other
        // link types for things such as a database accession number
        if let Some(ext_link_type) = &self.jats_ext_link_type {
            context.push_attr("ext-link-type", ext_link_type);
        } else if has_uri_scheme(&self.target) {
            context.push_attr("ext-link-type", "uri");
        }
        context.push_attr("xlink:href", &self.target);
        if let Some(title) = &self.title {
            context.push_attr("xlink:title", title);
        }
        self.content.to_jats(context);
        context.exit_elem();
    }
}

impl DomCodec for Link {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        // `target` and `id` properties are placed on the node element
        // for use by web components

        context.push_id(&self.id).push_attr("target", &self.target);

        // `target` (as `href`) and other standard HTML attributes put on inner <a> tag

        context.enter_elem_attrs("a", [("href", &self.target)]);

        if let Some(title) = &self.title {
            context.push_attr("title", title);
        }

        if let Some(rel) = &self.rel {
            context.push_attr("rel", rel);
        }

        context
            .push_slot_fn("span", "content", |context| self.content.to_dom(context))
            .exit_elem()
            .exit_node();
    }
}

impl LatexCodec for Link {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let command = if self.target.starts_with("https://") || self.target.starts_with("http://") {
            if self.content.is_empty() || to_text(&self.content) == self.target {
                "url"
            } else {
                "href"
            }
        } else if context.has_format_via_pandoc() {
            // Pandoc’s built-in LaTeX reader doesn’t implement Hyperref’s \autoref prefix logic,
            // so both \ref and \autoref get treated as “just the label number” when you go to DOCX.
            // See https://github.com/jgm/pandoc/issues/7463.
            // Therefore, we use \hyperref here as it allows us to set the content of the link (which
            // will have been done by Stencila's compile/link phases)
            "hyperref"
        } else if self.label_only.unwrap_or_default() {
            "ref"
        } else {
            "autoref"
        };

        if command == "hyperref" {
            context
                .char('\\')
                .str(command)
                .char('[')
                .property_str(NodeProperty::Target, self.target.trim_start_matches("#"))
                .char(']');
        } else {
            context
                .command_begin(command)
                .property_str(NodeProperty::Target, self.target.trim_start_matches("#"))
                .command_end();
        }

        if (command == "href" && !self.content.is_empty()) || command == "hyperref" {
            context
                .char('{')
                .property_fn(NodeProperty::Content, |context| {
                    self.content.to_latex(context)
                })
                .char('}');
        }

        context.exit_node();
    }
}

impl MarkdownCodec for Link {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, rel));

        // If the content is equal to the target and no title then only encode the content (i.e. an "autolink")
        // (it is better to encode the content and get a mapping entry for that than the target property)
        if let (1, Some(Inline::Text(content)), None) =
            (self.content.len(), self.content.first(), &self.title)
            && content.value.string == self.target
            && (self.target.starts_with("http://") || self.target.starts_with("https://"))
        {
            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .exit_node();
            return;
        }

        context
            .push_str("[")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str("](")
            .push_prop_str(NodeProperty::Target, &self.target);

        if let Some(title) = &self.title {
            context
                .push_str(" \"")
                .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context))
                .push_str("\"");
        }

        context.push_str(")").exit_node();
    }
}
