use stencila_codec_info::lost_options;

use crate::{SuggestionInline, SuggestionType, implem::suggestion::suggestion_attrs, prelude::*};

impl MarkdownCodec for SuggestionInline {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(
            context.mode,
            MarkdownEncodeMode::Clean | MarkdownEncodeMode::Render
        ) || !matches!(context.format, Format::Smd)
        {
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

        context.push_str(close);

        if let Some(attrs) = suggestion_attrs(&self.authors, &self.date_published) {
            context.push_str(&attrs);
        }

        context.exit_node();
    }
}
