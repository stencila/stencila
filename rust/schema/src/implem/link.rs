use stencila_codec_info::lost_options;
use stencila_codec_text_trait::to_text;

use crate::{Inline, Link, prelude::*};

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
