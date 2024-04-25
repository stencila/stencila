use codec_info::lost_options;
use codec_json5_trait::Json5Codec;

use crate::{
    prelude::*, ArrayValidator, BooleanValidator, ConstantValidator, DateTimeValidator,
    DateValidator, DurationValidator, EnumValidator, IntegerValidator, Node, NumberValidator,
    StringValidator, TimeValidator, TimestampValidator, TupleValidator,
};

/// Convert a node to a string
pub fn node_to_md(node: &Node) -> String {
    match node {
        Node::String(node) => string_to_md(node),
        _ => node.to_json5().unwrap_or_default(),
    }
}

/// Convert a string to a quoted string
pub fn string_to_md(string: &str) -> String {
    if string.contains('"') {
        ["'", string, "'"].concat()
    } else {
        ["\"", string, "\""].concat()
    }
}

impl MarkdownCodec for ArrayValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(
                self,
                id,
                items_nullable,
                items_validator,
                contains,
                min_items,
                max_items,
                unique_items
            ))
            .exit_node();
    }
}

impl MarkdownCodec for BooleanValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .exit_node();
    }
}

impl MarkdownCodec for ConstantValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .add_loss("ConstantValidator.value")
            .exit_node();
    }
}

impl MarkdownCodec for DateTimeValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(minimum) = &self.minimum {
            context
                .push_str(" min=")
                .push_prop_str(NodeProperty::Minimum, &string_to_md(&minimum.value));
        }

        if let Some(maximum) = &self.maximum {
            context
                .push_str(" max=")
                .push_prop_str(NodeProperty::Maximum, &string_to_md(&maximum.value));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for DateValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(minimum) = &self.minimum {
            context
                .push_str(" min=")
                .push_prop_str(NodeProperty::Minimum, &string_to_md(&minimum.value));
        }

        if let Some(maximum) = &self.maximum {
            context
                .push_str(" max=")
                .push_prop_str(NodeProperty::Maximum, &string_to_md(&maximum.value));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for DurationValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, minimum, maximum, time_units))
            .exit_node();
    }
}

impl MarkdownCodec for EnumValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_str(" vals=")
            .push_prop_str(
                NodeProperty::Values,
                &self.values.to_json5().unwrap_or_default(),
            )
            .exit_node();
    }
}

impl MarkdownCodec for IntegerValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(min) = &self.minimum {
            context
                .push_str(" min=")
                .push_prop_str(NodeProperty::Minimum, &min.to_string());
        }

        if let Some(emin) = &self.exclusive_minimum {
            context
                .push_str(" emin=")
                .push_prop_str(NodeProperty::ExclusiveMinimum, &emin.to_string());
        }

        if let Some(max) = &self.maximum {
            context
                .push_str(" max=")
                .push_prop_str(NodeProperty::Maximum, &max.to_string());
        }

        if let Some(emax) = &self.exclusive_maximum {
            context
                .push_str(" emax=")
                .push_prop_str(NodeProperty::ExclusiveMaximum, &emax.to_string());
        }

        if let Some(mult) = &self.multiple_of {
            context
                .push_str(" mult=")
                .push_prop_str(NodeProperty::MultipleOf, &mult.to_string());
        }

        context.exit_node();
    }
}

impl MarkdownCodec for NumberValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(min) = &self.minimum {
            context
                .push_str(" min=")
                .push_prop_str(NodeProperty::Minimum, &min.to_string());
        }

        if let Some(emin) = &self.exclusive_minimum {
            context
                .push_str(" emin=")
                .push_prop_str(NodeProperty::ExclusiveMinimum, &emin.to_string());
        }

        if let Some(max) = &self.maximum {
            context
                .push_str(" max=")
                .push_prop_str(NodeProperty::Maximum, &max.to_string());
        }

        if let Some(emax) = &self.exclusive_maximum {
            context
                .push_str(" emax=")
                .push_prop_str(NodeProperty::ExclusiveMaximum, &emax.to_string());
        }

        if let Some(mult) = &self.multiple_of {
            context
                .push_str(" mult=")
                .push_prop_str(NodeProperty::MultipleOf, &mult.to_string());
        }

        context.exit_node();
    }
}

impl MarkdownCodec for StringValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(min) = &self.min_length {
            context
                .push_str(" minlen=")
                .push_prop_str(NodeProperty::MinLength, &min.to_string());
        }

        if let Some(max) = &self.max_length {
            context
                .push_str(" maxlen=")
                .push_prop_str(NodeProperty::MaxLength, &max.to_string());
        }

        if let Some(pattern) = &self.pattern {
            context
                .push_str(" pattern=")
                .push_prop_str(NodeProperty::Pattern, &string_to_md(pattern));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for TimeValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if let Some(minimum) = &self.minimum {
            context
                .push_str(" min=")
                .push_prop_str(NodeProperty::Minimum, &string_to_md(&minimum.value));
        }

        if let Some(maximum) = &self.maximum {
            context
                .push_str(" max=")
                .push_prop_str(NodeProperty::Maximum, &string_to_md(&maximum.value));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for TimestampValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, minimum, maximum, time_units))
            .exit_node();
    }
}

impl MarkdownCodec for TupleValidator {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, items))
            .exit_node();
    }
}
