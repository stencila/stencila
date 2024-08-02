use codec_info::lost_options;

use crate::{prelude::*, transforms::blocks_to_inlines, Figure, ImageObject, Inline};

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

        context.enter_elem_attrs("figure", [("slot", "content")]);

        self.content.to_dom(context);

        if let Some(caption) = &self.caption {
            context.push_slot_fn("figcaption", "caption", |context| {
                caption_to_dom(context, "figure-label", "Figure", &self.label, caption)
            });
        }

        context.exit_elem().exit_node();
    }
}

impl MarkdownCodec for Figure {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                ':',
                "figure",
                |context| {
                    let inlines = blocks_to_inlines(self.content.clone());
                    let mut urls = inlines.iter().filter_map(|inline| match inline {
                        Inline::ImageObject(ImageObject { content_url, .. }) => Some(content_url),
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
            );
        } else {
            context.push_colons().push_str(" figure");

            if !self.label_automatically.unwrap_or(true) {
                if let Some(label) = &self.label {
                    context.push_str(" ");
                    context.push_prop_str(NodeProperty::Label, label);
                }
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
                .newline();
        }

        context.exit_node().newline();
    }
}
