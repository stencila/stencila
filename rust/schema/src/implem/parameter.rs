use codec_html_trait::encode::{attr, elem};

use codec_info::{lost_exec_options, lost_options};
use common::inflector::Inflector;

use crate::{prelude::*, Parameter, Validator};

use super::validators::node_to_md;

impl Parameter {
    pub fn to_html_special(&self, _context: &mut HtmlEncodeContext) -> String {
        let mut attrs = vec![];
        let mut children = vec![];

        if let Some(id) = &self.id {
            attrs.push(attr("id", id));
        }

        let label = elem(
            "label",
            &[attr("for", &self.name)],
            &[self.name.to_title_case()],
        );
        children.push(label);

        let input = elem("input", &[attr("name", &self.name)], &[]);
        children.push(input);

        elem("stencila-parameter", &attrs, &children)
    }
}

impl MarkdownCodec for Parameter {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self))
            .push_str("&[")
            .push_prop_str("name", &self.name)
            .push_str("]");

        // Return early if no attributes to add
        if self.value.is_none()
            && self.options.default.is_none()
            && self.options.validator.is_none()
        {
            context.exit_node();
            return;
        }

        context.push_str("{");

        if let Some(validator) = &self.options.validator {
            use Validator::*;
            let name = match validator {
                ArrayValidator(..) => "array",
                BooleanValidator(..) => "bool",
                ConstantValidator(..) => "const",
                DateTimeValidator(..) => "datetime",
                DateValidator(..) => "date",
                DurationValidator(..) => "duration",
                EnumValidator(..) => "enum",
                IntegerValidator(..) => "int",
                NumberValidator(..) => "num",
                StringValidator(..) => "str",
                TimestampValidator(..) => "timestamp",
                TimeValidator(..) => "time",
                TupleValidator(..) => "tuple",
            };
            context.push_prop_str("validator", name);
        }

        if let Some(val) = &self.options.default {
            context
                .push_str(" def=")
                .push_prop_str("default", &node_to_md(val));
        }

        if let Some(validator) = &self.options.validator {
            context.push_prop_fn("validator", |context| validator.to_markdown(context));
        }

        context.push_str("}").exit_node();
    }
}
