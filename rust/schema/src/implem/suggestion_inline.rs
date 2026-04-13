use stencila_codec_info::lost_options;

use crate::{SuggestionInline, SuggestionType, prelude::*};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.mode, MarkdownEncodeMode::Clean) {
            context.enter_node(self.node_type(), self.node_id());

            match self.suggestion_type {
                Some(SuggestionType::Delete) => {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context);
                    });
                }
                Some(SuggestionType::Replace) => {
                    context.push_prop_fn(NodeProperty::Original, |context| {
                        self.original.to_markdown(context);
                    });
                }
                _ => {}
            }

            context.exit_node();
            return;
        }

        // If rendering, or format is anything other than Stencila Markdown, skip encoding
        // and record as loss
        if matches!(context.mode, MarkdownEncodeMode::Render)
            || !matches!(context.format, Format::Smd)
        {
            context.losses.add(self.node_type().to_string());

            return;
        }

        let (open, close) = match self.suggestion_type {
            Some(SuggestionType::Delete) => ("{--", "--}"),
            Some(SuggestionType::Replace) => ("{~~", "~~}"),
            _ => ("{++", "++}"),
        };

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str(open);

        match self.suggestion_type {
            Some(SuggestionType::Replace) => {
                context
                    .push_prop_fn(NodeProperty::Original, |context| {
                        self.original.to_markdown(context)
                    })
                    .push_str("~>")
                    .push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    });
            }
            _ => {
                context.push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                });
            }
        }

        context.push_str(close).exit_node();
    }
}
