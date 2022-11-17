use std::string::ToString;

use codec::common::tracing;

use node_dispatch::dispatch_validator;
use stencila_schema::*;
use suids::Suid;

use super::{attr_id, elem, elem_slot, EncodeContext, ToHtml};

impl ToHtml for ValidatorTypes {
    /// Encode a `ValidatorTypes` variant to HTML
    ///
    /// Simply dispatches to one of the concrete validator types.
    fn to_html(&self, context: &mut EncodeContext) -> String {
        dispatch_validator!(self, to_html, context)
    }
}

/**
 * Generate a HTML tag name for a validator
 *
 * Used when a validator is used as an optional property and that
 * property is none so that we can provider a placeholder element.
 */
pub(crate) fn validator_tag_name(validator: Option<&ValidatorTypes>) -> String {
    let validator = match validator {
        Some(validator) => validator,
        None => return "stencila-validator".to_string(),
    };

    use ValidatorTypes::*;
    match validator {
        Validator(..) => "stencila-validator",
        ArrayValidator(..) => "stencila-array-validator",
        BooleanValidator(..) => "stencila-boolean-validator",
        ConstantValidator(..) => "stencila-constant-validator",
        DateTimeValidator(..) => "stencila-dateTime-validator",
        DateValidator(..) => "stencila-date-validator",
        DurationValidator(..) => "stencila-duration-validator",
        EnumValidator(..) => "stencila-enum-validator",
        IntegerValidator(..) => "stencila-integer-validator",
        NumberValidator(..) => "stencila-number-validator",
        StringValidator(..) => "stencila-string-validator",
        TimeValidator(..) => "stencila-time-validator",
        TimestampValidator(..) => "stencila-timestamp-validator",
        TupleValidator(..) => "stencila-tuple-validator",
    }
    .to_string()
}

impl ToHtml for Validator {
    /// Encode a `Validator` to HTML
    ///
    /// Note that this is just an empty base for all other validators and should not
    /// really be part of the `ValidatorTypes` enum and never be instantiated.
    /// So this just logs a warning returns an empty string.
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        tracing::warn!("Unexpected instantiation of `Validator` type");
        String::new()
    }
}

impl ToHtml for ConstantValidator {
    /// Encode a `ConstantValidator` to HTML
    ///
    /// Encodes the `value` (a `Node`) as a JSON attribute.
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-constant-validator",
            &[attr_id(&self.id), self.value.to_attr("value")],
            "",
        )
    }
}

impl ToHtml for EnumValidator {
    /// Encode a `EnumValidator` to HTML
    ///
    /// Encodes the `values` (a vector of `Nodes`) as a JSON attribute since they are not
    /// intended to be viewed by the user directly.
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-enum-validator",
            &[attr_id(&self.id), self.values.to_attr("values")],
            "",
        )
    }
}

impl ToHtml for BooleanValidator {
    /// Encode a `BooleanValidator` to HTML
    ///
    /// No properties, so just an empty element used to indicate the type
    /// Note: don't use `elem_empty` because don't want a self closing tag here!
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem("stencila-boolean-validator", &[attr_id(&self.id)], "")
    }
}

/**
 * Encode the properties of an `IntegerValidator` or `NumberValidator` as a vector of attributes
 */
#[allow(clippy::box_collection)]
fn numeric_validator_attrs(
    id: &Option<Suid>,
    minimum: &Option<Number>,
    exclusive_minimum: &Option<Number>,
    maximum: &Option<Number>,
    exclusive_maximum: &Option<Number>,
    multiple_of: &Option<Number>,
) -> Vec<String> {
    vec![
        id.to_attr("id"),
        minimum.to_attr("minimum"),
        exclusive_minimum.to_attr("exclusive-minimum"),
        maximum.to_attr("maximum"),
        exclusive_maximum.to_attr("exclusive-maximum"),
        multiple_of.to_attr("multiple-of"),
    ]
}

impl ToHtml for IntegerValidator {
    /// Encode a `IntegerValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-integer-validator",
            &numeric_validator_attrs(
                &self.id,
                &self.minimum,
                &self.exclusive_minimum,
                &self.maximum,
                &self.exclusive_maximum,
                &self.multiple_of,
            ),
            "",
        )
    }
}

impl ToHtml for NumberValidator {
    /// Encode a `NumberValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-number-validator",
            &numeric_validator_attrs(
                &self.id,
                &self.minimum,
                &self.exclusive_minimum,
                &self.maximum,
                &self.exclusive_maximum,
                &self.multiple_of,
            ),
            "",
        )
    }
}

impl ToHtml for StringValidator {
    /// Encode a `StringValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-string-validator",
            &[
                attr_id(&self.id),
                self.min_length.to_attr("min-length"),
                self.max_length.to_attr("max-length"),
                self.pattern.to_attr("pattern"),
            ],
            "",
        )
    }
}

impl ToHtml for DateValidator {
    /// Encode a `DateValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-date-validator",
            &[
                attr_id(&self.id),
                self.minimum.to_attr("minimum"),
                self.minimum.to_attr("maximum"),
            ],
            "",
        )
    }
}

impl ToHtml for TimeValidator {
    /// Encode a `TimeValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-time-validator",
            &[
                attr_id(&self.id),
                self.minimum.to_attr("minimum"),
                self.minimum.to_attr("maximum"),
            ],
            "",
        )
    }
}

impl ToHtml for DateTimeValidator {
    /// Encode a `DateTimeValidator` to HTML
    ///
    /// All properties are `Primitive`s so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-datetime-validator",
            &[
                attr_id(&self.id),
                self.minimum.to_attr("minimum"),
                self.minimum.to_attr("maximum"),
            ],
            "",
        )
    }
}

impl ToHtml for TimestampValidator {
    /// Encode a `TimestampValidator` to HTML
    ///
    /// All properties are `Primitive` or enums so can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-timestamp-validator",
            &[
                attr_id(&self.id),
                self.minimum.to_attr("minimum"),
                self.minimum.to_attr("maximum"),
                self.time_units.to_attr("time-units"),
            ],
            "",
        )
    }
}

impl ToHtml for DurationValidator {
    /// Encode a `DurationValidator` to HTML
    ///
    /// All properties are `Primitive` or enums can be encoded as attributes
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "stencila-duration-validator",
            &[
                attr_id(&self.id),
                self.minimum.to_attr("minimum"),
                self.minimum.to_attr("maximum"),
                self.time_units.to_attr("time-units"),
            ],
            "",
        )
    }
}

impl ToHtml for ArrayValidator {
    /// Encode an `ArrayValidator` to HTML
    ///
    /// Encodes the properties that are themselves validators as
    /// elements and other other, primitive properties, as attributes.
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let items_validator = elem_slot(
            "stencila-validator",
            "items-validator",
            &self.items_validator,
            context,
        );
        let contains = elem_slot("stencila-validator", "contains", &self.contains, context);

        elem(
            "stencila-array-validator",
            &[
                attr_id(&self.id),
                self.max_items.to_attr("max_items"),
                self.min_items.to_attr("min_items"),
                self.items_nullable.to_attr("items-nullable"),
                self.unique_items.to_attr("unique-items"),
            ],
            &[items_validator, contains].concat(),
        )
    }
}

impl ToHtml for TupleValidator {
    /// Encode a `TupleValidator` to HTML
    ///
    /// Encodes each of the validators in `items` to HTML
    fn to_html(&self, context: &mut EncodeContext) -> String {
        elem(
            "stencila-tuple-validator",
            &[attr_id(&self.id)],
            &self.items.to_html(context),
        )
    }
}
