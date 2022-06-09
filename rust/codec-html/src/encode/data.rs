//! HTML encoding for node types in the "data" category
//!
//! ## Validators
//!
//! Currently validators are used for the `validator` property of two node
//! types: `DatatableColumn` and `Parameter`. This modules provides to functions
//! for each of these use cases.
//!
//! For use in a `DatatableColumn`, the usual `ToHtml` trait is implemented which
//! generates a custom element e.g. `<stencila-number-validator>` to indicate the
//! type of the column and it's restrictions.
//!
//! For use in a `Parameter`, the `ToAttrs` trait is implemented so that the validator
//! can be represented as standard HTML form input attributes e.g. `type="number" minimum="0"`

use std::string::ToString;

use codec::common::tracing;
use codec_txt::ToTxt;
use node_dispatch::dispatch_validator;
use stencila_schema::*;

use super::{
    attr, attr_bool, attr_id, attr_itemprop, attr_itemtype, attr_itemtype_str, attr_prop,
    attr_slot, concat, concat_html, elem, elem_empty, elem_meta, elem_placeholder, nothing,
    EncodeContext, ToHtml,
};

/// Encode a `Datatable`
impl ToHtml for Datatable {
    fn to_html(&self, context: &EncodeContext) -> String {
        let columns = elem(
            "tr",
            &[attr_prop("columns")],
            &concat_html(&self.columns, context),
        );
        let rows = elem_meta("rows", "");
        let values = elem_meta("values", "");

        let head = elem("thead", &[], &[columns, rows, values].concat());

        let rows = self.columns.iter().fold(0, |mut rows, column| {
            let len = column.values.len();
            if len > rows {
                rows = len
            }
            rows
        });
        let rows = (0..rows)
            .into_iter()
            .map(|row| {
                let data = concat(&self.columns, |column| {
                    let data = if let Some(data) = column.values.get(row) {
                        data.to_html(context)
                    } else {
                        nothing()
                    };
                    elem("td", &[], &data)
                });
                elem("tr", &[], &data)
            })
            .collect::<Vec<String>>()
            .concat();
        let body = elem("tbody", &[], &rows);

        elem(
            "stencila-datatable",
            &[attr_itemtype::<Self>()],
            &elem("table", &[], &[head, body].concat()),
        )
    }
}

/// Encode a `DatatableColumn`
impl ToHtml for DatatableColumn {
    fn to_html(&self, context: &EncodeContext) -> String {
        let name = elem("span", &[attr_prop("name")], &self.name.to_html(context));
        elem("th", &[attr_itemtype::<Self>()], &[name].concat())
    }
}

/// Encode a `Parameter`
impl ToHtml for Parameter {
    fn to_html(&self, context: &EncodeContext) -> String {
        // Generate a unique id for the <input> to be able to associate the
        // <label> with it. We avoid using `self.name` or `self.name` which could
        // get updated via patches (and thus would need changing in two places).
        // But for determinism in tests, create a static id.
        let input_id = match cfg!(test) {
            true => "input-id".to_string(),
            false => uuids::generate("in").to_string(),
        };

        let name = elem(
            "label",
            &[attr_prop("name"), attr_slot("name"), attr("for", &input_id)],
            &self.name,
        );

        // Meta elements for `validator`, `default`, and `value` that add HTML Microdata and
        // are used as "proxies" to the attributes added to the <input> element when patching the DOM

        let validator = elem_empty(
            "meta",
            &[
                attr_itemprop("validator"),
                self.validator
                    .as_deref()
                    .map_or_else(nothing, |node| attr_itemtype_str(node.as_ref())),
            ],
        );

        let default = elem_empty(
            "meta",
            &[
                attr_itemprop("default"),
                self.default
                    .as_deref()
                    .map_or_else(nothing, |node| attr_itemtype_str(node.as_ref())),
                self.default
                    .as_deref()
                    .map_or_else(nothing, |node| attr("content", &node.to_txt())),
            ],
        );

        let value = elem_empty(
            "meta",
            &[
                attr_itemprop("value"),
                self.value
                    .as_deref()
                    .map_or_else(nothing, |node| attr_itemtype_str(node.as_ref())),
                self.value
                    .as_deref()
                    .map_or_else(nothing, |node| attr("content", &node.to_txt())),
            ],
        );

        let input = if let Some(ValidatorTypes::EnumValidator(validator)) =
            self.validator.as_deref()
        {
            // Select the `value`, or secondarily, the `default` <option>
            let value = self
                .value
                .as_deref()
                .or_else(|| self.default.as_deref())
                .map(|node| node.to_txt())
                .unwrap_or_default();

            let options = match validator.values.as_deref() {
                Some(values) => concat(values, |node| {
                    let txt = node.to_txt();
                    let selected = if txt == value { "selected" } else { "" };
                    elem("option", &[attr("value", &txt), selected.to_string()], &txt)
                }),
                None => nothing(),
            };

            elem(
                "select",
                &[attr("id", &input_id), attr_slot("value")],
                &[options].concat(),
            )
        } else {
            // Get the attrs corresponding to the validator so that we
            // can add them to the <input> element
            let validator_attrs = match &self.validator {
                Some(validator) => validator.to_attrs(context),
                None => vec![attr("type", "text")],
            };

            // If the parameter's `default` property is set then set a `placeholder` attribute
            let placeholder_attr = match &self.default {
                Some(node) => attr("placeholder", &node.to_txt()),
                None => "".to_string(),
            };

            let value_attr = match &self.value {
                Some(node) => attr("value", &node.to_txt()),
                None => "".to_string(),
            };

            // If a `BooleanValidator` then need to set the `checked` attribute if true
            let checked_attr =
                if let (Some(ValidatorTypes::BooleanValidator(..)), Some(Node::Boolean(true))) =
                    (self.validator.as_deref(), self.value.as_deref())
                {
                    attr("checked", "")
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
                    checked_attr,
                ],
            )
        };

        elem(
            "stencila-parameter",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &[name, validator, default, value, input].concat(),
        )
    }
}

/// Encode a `ValidatorTypes` variant
impl ToHtml for ValidatorTypes {
    fn to_html(&self, context: &EncodeContext) -> String {
        dispatch_validator!(self, to_html, context)
    }

    fn to_attrs(&self, context: &EncodeContext) -> Vec<String> {
        dispatch_validator!(self, to_attrs, context)
    }
}

/// Encode a `Validator`
///
/// Note that this is just an empty base for all other validators and should not
/// really be part of the `ValidatorTypes` enum and never be instantiated.
/// So this just logs a warning returns an empty string.
impl ToHtml for Validator {
    fn to_html(&self, _context: &EncodeContext) -> String {
        tracing::warn!("Unexpected instantiation of `Validator` type");
        String::new()
    }
}

/// Encode a `ArrayValidator`
///
/// No properties, so just an empty element used to indicate the type
impl ToHtml for ArrayValidator {
    fn to_html(&self, _context: &EncodeContext) -> String {
        todo!()
    }
}

/// Encode a `BooleanValidator`
///
/// No properties, so just an empty element used to indicate the type
impl ToHtml for BooleanValidator {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem_empty(
            "stencila-boolean-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
        )
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        vec![attr("type", "checkbox")]
    }
}

/// Encode a `ConstantValidator`
///
/// Encodes the constant `value`.
impl ToHtml for ConstantValidator {
    fn to_html(&self, context: &EncodeContext) -> String {
        let value = elem(
            "span",
            &[attr_prop("value"), attr_slot("value")],
            &self.value.to_html(context),
        );
        elem(
            "stencila-constant-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &value,
        )
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        // The `type=text` could be changed to depend on the
        // `Node` type of the `value`.
        vec![attr("type", "text"), attr_bool("readonly")]
    }
}

/// Encode a `EnumValidator`
///
/// Encodes the possible `values`. Each of these will be an element
/// indicating the type of the value.
impl ToHtml for EnumValidator {
    fn to_html(&self, context: &EncodeContext) -> String {
        let values = elem(
            "div",
            &[attr_prop("values"), attr_slot("values")],
            &self.values.to_html(context),
        );
        elem(
            "stencila-enum-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &values,
        )
    }
}

/// Encode a `IntegerValidator`
///
/// No properties, so just an empty element used to indicate the type.
/// It is probable that in the future, this will have properties such as
/// `minimum` and `maximum` like `NumberValidator`.
impl ToHtml for IntegerValidator {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem_empty(
            "stencila-integer-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
        )
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        vec![attr("type", "number")]
    }
}

/// Encode a `NumberValidator`
///
/// Encodes all properties
impl ToHtml for NumberValidator {
    fn to_html(&self, context: &EncodeContext) -> String {
        // We use `.map(|value| value.to_string())` for properties so they get
        // rendered as text, not wrapped as a `<span itemtype="https://schema.org/Number"...`

        let minimum = elem_placeholder(
            "span",
            &[attr_prop("minimum"), attr_slot("minimum")],
            &self.minimum.map(|value| value.to_string()),
            context,
        );

        let exclusive_minimum = elem_placeholder(
            "span",
            &[
                attr_prop("exclusive_minimum"),
                attr_slot("exclusive-minimum"),
            ],
            &self.exclusive_minimum.map(|value| value.to_string()),
            context,
        );

        let maximum = elem_placeholder(
            "span",
            &[attr_prop("maximum"), attr_slot("maximum")],
            &self.maximum.map(|value| value.to_string()),
            context,
        );

        let exclusive_maximum = elem_placeholder(
            "span",
            &[
                attr_prop("exclusive_maximum"),
                attr_slot("exclusive-maximum"),
            ],
            &self.exclusive_maximum.map(|value| value.to_string()),
            context,
        );

        let multiple_of = elem_placeholder(
            "span",
            &[attr_prop("multiple_of"), attr_slot("multiple-of")],
            &self.multiple_of.map(|value| value.to_string()),
            context,
        );

        elem(
            "stencila-number-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &[
                minimum,
                exclusive_minimum,
                maximum,
                exclusive_maximum,
                multiple_of,
            ]
            .concat(),
        )
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        // See https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/number for
        // attributes supported here.
        let mut attrs = Vec::with_capacity(4);
        attrs.push(attr("type", "number"));
        if let Some(min) = self.minimum {
            attrs.push(attr("min", &min.to_string()))
        }
        if let Some(max) = self.maximum {
            attrs.push(attr("max", &max.to_string()))
        }
        if let Some(step) = self.multiple_of {
            attrs.push(attr("step", &step.to_string()))
        }
        attrs
    }
}

/// Encode a `StringValidator`
///
/// Encodes all properties
impl ToHtml for StringValidator {
    fn to_html(&self, context: &EncodeContext) -> String {
        let min_length = elem_placeholder(
            "span",
            &[attr_prop("min_length"), attr_slot("min-length")],
            &self.min_length.map(|value| value.to_string()),
            context,
        );

        let max_length = elem_placeholder(
            "span",
            &[attr_prop("max_length"), attr_slot("max-length")],
            &self.max_length.map(|value| value.to_string()),
            context,
        );

        let pattern = elem_placeholder(
            "span",
            &[attr_prop("pattern"), attr_slot("pattern")],
            &self.pattern,
            context,
        );

        elem(
            "stencila-string-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &[min_length, max_length, pattern].concat(),
        )
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        // See https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/text for
        // attributes supported here.
        let mut attrs = Vec::with_capacity(4);
        attrs.push(attr("type", "text"));
        if let Some(min_length) = self.min_length {
            attrs.push(attr("minlength", &min_length.to_string()))
        }
        if let Some(max_length) = self.max_length {
            attrs.push(attr("maxlength", &max_length.to_string()))
        }
        if let Some(pattern) = &self.pattern {
            attrs.push(attr("pattern", pattern))
        }
        attrs
    }
}

/// Encode a `TupleValidator`
///
/// Encodes each of the validators in `items`
impl ToHtml for TupleValidator {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "stencila-tuple-validator",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.items.to_html(context),
        )
    }
}
