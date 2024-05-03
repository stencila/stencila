use codec_info::lost_options;

use crate::{prelude::*, StyledInline};

impl DomCodec for StyledInline {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        self.code.to_dom_attr("code", context);

        if let Some(style_language) = &self.style_language {
            context.push_attr("style-language", style_language);
        }

        if !context.standalone {
            if let Some(css) = &self.options.css {
                context.push_attr("css", css);
            }
            if let Some(class_list) = &self.options.class_list {
                context.push_attr("class-list", class_list);
            }
        } else if let Some(css) = &self.options.css {
            context.push_css(css);
        };

        if let Some(messages) = &self.options.compilation_messages {
            context.push_slot_fn("span", "compilation-messages", |context| {
                messages.to_dom(context)
            });
        }

        if let Some(authors) = &self.options.authors {
            context.push_slot_fn("span", "authors", |context| authors.to_dom(context));
        }

        context.push_slot_fn("span", "content", |context| {
            if let Some(class) = &self.options.class_list {
                context.push_attr("class", class);
            };
            self.content.to_dom(context)
        });

        context.exit_node();
    }
}

impl MarkdownCodec for StyledInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, style_language))
            .merge_losses(lost_options!(
                self.options,
                compilation_digest,
                compilation_messages,
                css,
                class_list
            ))
            .push_str("[")
            .push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            })
            .push_str("]{")
            .push_prop_fn(NodeProperty::Code, |context| self.code.to_markdown(context))
            .push_str("}");

        context.exit_node();
    }
}
