use stencila_codec_info::lost_options;

use crate::{Figure, ImageObject, Inline, prelude::*, transforms::blocks_to_inlines};

use super::utils::caption_to_dom;

impl DomCodec for Figure {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        if let Some(id) = &self.id {
            context
                .enter_slot("div", "id")
                .push_attr("id", id)
                .exit_slot();
        }

        context.push_slot_fn("figure", "content", |context| {
            self.content.to_dom(context);

            if (self.label.is_some() && matches!(self.label_automatically, Some(false)))
                || self.caption.is_some()
            {
                // The HTML spec requires <figcaption> to be within <figure>. But slotted elements must be direct children
                // of the custom element (in this case, <stencila-figure>). For those reasons, the caption is not
                // assigned to a slot
                context.enter_elem("figcaption");
                caption_to_dom(
                    context,
                    "figure-label",
                    "Figure",
                    &self.label,
                    &self.caption,
                );
                context.exit_elem();
            }
        });

        context.exit_node();
    }
}

impl MarkdownCodec for Figure {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        if matches!(context.format, Format::Myst) {
            context
                .myst_directive(
                    ':',
                    "figure",
                    |context| {
                        let inlines = blocks_to_inlines(self.content.clone());
                        let mut urls = inlines.iter().filter_map(|inline| match inline {
                            Inline::ImageObject(ImageObject { content_url, .. }) => {
                                Some(content_url)
                            }
                            _ => None,
                        });
                        if let Some(url) = urls.next() {
                            context.push_str(" ").push_str(url);
                        }
                    },
                    |context| {
                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }
                    },
                    |context| {
                        if let Some(caption) = &self.caption {
                            caption.to_markdown(context);
                        }
                    },
                )
                .exit_node()
                .newline();
        } else {
            context.push_colons().push_str(" figure");

            if !self.label_automatically.unwrap_or(true)
                && let Some(label) = &self.label
            {
                context.push_str(" ");
                context.push_prop_str(NodeProperty::Label, label);
            }

            context.push_str("\n\n").increase_depth();

            if let Some(caption) = &self.caption {
                context.push_prop_fn(NodeProperty::Caption, |context| {
                    caption.to_markdown(context)
                });
            }

            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .decrease_depth()
                .push_colons()
                .newline()
                .exit_node()
                .newline();
        }
    }
}
