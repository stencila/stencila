use codec_info::lost_options;

use crate::{prelude::*, Inline, Link};

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

impl MarkdownCodec for Link {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, rel));

        // If the content is equal to the target and no title then only encode the content (i.e. an "autolink")
        // (it is better to encode the content and get a mapping entry for that than the target property)
        if let (1, Some(Inline::Text(content)), None) =
            (self.content.len(), self.content.first(), &self.title)
        {
            if content.value.string == self.target {
                context
                    .push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    })
                    .exit_node();
                return;
            }
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
