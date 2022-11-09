use std::string::ToString;

use codec::{common::serde_json, EncodeMode};
use codec_txt::ToTxt;

use stencila_schema::*;

use super::{
    attr, attr_id, attr_prop, attr_slot, concat, elem, elem_empty, elem_placeholder, elem_slot,
    nothing, validators::validator_tag_name, EncodeContext, ToHtml,
};

/// Encode a `Parameter`
impl ToHtml for Parameter {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let name = self.name.to_attr("name");
        let label = self.label.to_attr("label");
        let default = self.default.to_attr("default");
        let value = self.value.to_attr("value");
        let derived_from = self.derived_from.to_attr("derived-from");
        let hidden = self.hidden.to_attr("hidden");

        let errors = elem_placeholder("span", &[attr_slot("errors")], &self.errors, context);

        let validator = elem_slot(
            &validator_tag_name(self.validator.as_deref()),
            "validator",
            &self.validator,
            context,
        );

        elem(
            "stencila-parameter",
            &[
                attr_id(&self.id),
                name,
                label,
                default,
                value,
                derived_from,
                hidden,
            ],
            &[String::new(), errors, validator].concat(),
        )
    }
}

pub(crate) fn label_and_input(
    name: &str,
    validator: &Option<Box<ValidatorTypes>>,
    value: &Option<Box<Node>>,
    default: &Option<Box<Node>>,
    context: &mut EncodeContext,
) -> (String, String) {
    // Generate a unique id for the <input> to be able to associate the
    // <label> with it. We avoid using `self.id` or `self.name` which could
    // get updated via patches (and thus would need changing in two places).
    // But for determinism in tests, create a static id.
    let input_id = match cfg!(test) {
        true => "input-id".to_string(),
        false => uuids::generate("in").to_string(),
    };

    let label = elem(
        "label",
        &[attr_prop("name"), attr_slot("name"), attr("for", &input_id)],
        name,
    );

    let input = if let Some(ValidatorTypes::EnumValidator(validator)) = validator.as_deref() {
        // Select the `value`, or secondarily, the `default` <option>
        let value = value
            .as_deref()
            .or(default.as_deref())
            .map(|node| node.to_txt())
            .unwrap_or_default();

        let options = concat(&validator.values, |node| {
            let txt = node.to_txt();
            let selected = if txt == value { "selected" } else { "" };
            elem("option", &[attr("value", &txt), selected.to_string()], &txt)
        });

        elem(
            "select",
            &[attr("id", &input_id), attr_slot("value")],
            &[options].concat(),
        )
    } else {
        // Get the attrs corresponding to the validator so that we
        // can add them to the <input> element
        let validator_attrs = match &validator {
            Some(_validator) => Vec::new(), //validator.to_attrs(context),
            None => vec![attr("type", "text")],
        };

        fn node_to_attr_value(node: &Node) -> String {
            match node {
                Node::Null(node) => node.to_string(),
                Node::Boolean(node) => node.to_string(),
                Node::Integer(node) => node.to_string(),
                Node::Number(node) => node.to_string(),
                Node::String(node) => node.to_string(),
                Node::Date(node) => node.to_string(),
                Node::Time(node) => node.to_string(),
                Node::DateTime(node) => node.to_string(),
                _ => serde_json::to_string(node).unwrap_or_else(|_| "null".to_string()),
            }
        }

        // If the parameter's `default` property is set then set a `placeholder` attribute
        let placeholder_attr = match &default {
            Some(node) => attr("placeholder", &node_to_attr_value(node.as_ref())),
            None => "".to_string(),
        };

        let value_attr = match &value {
            Some(node) => attr("value", &node_to_attr_value(node.as_ref())),
            None => "".to_string(),
        };

        // Add a size attribute which will expand the horizontal with of the input to match the content.
        // This is useful when generating RPNGs to avoid extra whitespace. There is not an easy way to do this
        // using CSS, see https://css-tricks.com/auto-growing-inputs-textareas/
        let size_attr = value
            .as_ref()
            .or(default.as_ref())
            .map(|node| {
                attr(
                    "size",
                    &(node_to_attr_value(node.as_ref()).len() + 1).to_string(),
                )
            })
            .unwrap_or_default();

        // If a `BooleanValidator` then need to set the `checked` attribute if true
        let checked_attr =
            if let (Some(ValidatorTypes::BooleanValidator(..)), Some(Node::Boolean(true))) =
                (validator.as_deref(), value.as_deref())
            {
                attr("checked", "")
            } else {
                nothing()
            };

        let disabled_attr = if context.options.mode < EncodeMode::Interact {
            "disabled".to_string()
        } else {
            nothing()
        };

        elem_empty(
            "input",
            &[
                attr("id", &input_id),
                attr_slot("value"),
                validator_attrs.join(" "),
                placeholder_attr,
                value_attr,
                size_attr,
                checked_attr,
                disabled_attr,
            ],
        )
    };

    (label, input)
}
