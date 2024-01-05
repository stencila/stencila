use codec_html_trait::encode::{attr, elem};
use codec_json5_trait::Json5Codec;
use common::inflector::Inflector;

use crate::{prelude::*, IntegerValidator, Node, NumberValidator, Parameter, Validator};

impl Parameter {
    pub fn to_html_special(&self) -> String {
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

    pub fn to_markdown_special(&self, _context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut md = ["&[", &self.name, "]"].concat();
        let mut losses = Losses::none();

        /// Convert a node to a string
        ///
        /// Uses `to_json5` except for strings containing double
        /// quotes for which JSON5 can not be used because it uses backslash
        /// escapes which are also used by Markdown.
        fn node_to_md(node: &Node) -> String {
            match node {
                Node::String(node) => string_to_md(node),
                _ => node.to_json5().unwrap_or_default(),
            }
        }
        fn string_to_md(string: &str) -> String {
            if string.contains('"') {
                ["'", string, "'"].concat()
            } else {
                ["\"", string, "\""].concat()
            }
        }

        let attr_value = |attrs: &mut String| {
            if let Some(val) = &self.value {
                attrs.push_str(" val=");
                attrs.push_str(&node_to_md(val));
            }
        };

        let attr_default = |attrs: &mut String| {
            if let Some(def) = &self.options.default {
                attrs.push_str(" def=");
                attrs.push_str(&node_to_md(def));
            }
        };

        // A macro for date/time validators
        macro_rules! attrs_min_max {
            ($name:expr, $validator:expr) => {{
                let mut attrs = $name.to_string();
                attr_value(&mut attrs);
                attr_default(&mut attrs);

                if let Some(min) = $validator.minimum.as_ref().map(|min| &min.value) {
                    attrs.push_str(" min=");
                    attrs.push_str(&min.to_json5().unwrap_or_default());
                }

                if let Some(max) = $validator.maximum.as_ref().map(|max| &max.value) {
                    attrs.push_str(" max=");
                    attrs.push_str(&max.to_json5().unwrap_or_default());
                }

                attrs
            }};
        }

        if let Some(validator) = &self.options.validator {
            let attrs = match validator {
                Validator::BooleanValidator(..) => {
                    let mut attrs = "bool".to_string();

                    attr_value(&mut attrs);
                    attr_default(&mut attrs);

                    attrs
                }

                Validator::IntegerValidator(IntegerValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..
                })
                | Validator::NumberValidator(NumberValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..
                }) => {
                    let mut attrs = match validator {
                        Validator::IntegerValidator(..) => "int",
                        Validator::NumberValidator(..) => "num",
                        _ => unreachable!(),
                    }
                    .to_string();

                    attr_value(&mut attrs);
                    attr_default(&mut attrs);

                    if let Some(min) = &minimum {
                        attrs.push_str(" min=");
                        attrs.push_str(&min.to_string());
                    }

                    if let Some(emin) = &exclusive_minimum {
                        attrs.push_str(" emin=");
                        attrs.push_str(&emin.to_string());
                    }

                    if let Some(max) = &maximum {
                        attrs.push_str(" max=");
                        attrs.push_str(&max.to_string());
                    }

                    if let Some(emax) = &exclusive_maximum {
                        attrs.push_str(" emax=");
                        attrs.push_str(&emax.to_string());
                    }

                    if let Some(mult) = &multiple_of {
                        attrs.push_str(" mult=");
                        attrs.push_str(&mult.to_string());
                    }

                    attrs
                }

                Validator::StringValidator(validator) => {
                    let mut attrs = "str".to_string();

                    attr_value(&mut attrs);
                    attr_default(&mut attrs);

                    if let Some(min) = &validator.min_length {
                        attrs.push_str(" min=");
                        attrs.push_str(&min.to_string());
                    }

                    if let Some(max) = &validator.max_length {
                        attrs.push_str(" max=");
                        attrs.push_str(&max.to_string());
                    }

                    if let Some(pattern) = &validator.pattern {
                        attrs.push_str(" pattern=");
                        attrs.push_str(&string_to_md(pattern));
                    }

                    attrs
                }

                Validator::DateValidator(validator) => {
                    attrs_min_max!("date", validator)
                }
                Validator::TimeValidator(validator) => {
                    attrs_min_max!("time", validator)
                }
                Validator::DateTimeValidator(validator) => {
                    attrs_min_max!("datetime", validator)
                }

                Validator::EnumValidator(validator) => {
                    let mut attrs = "enum".to_string();

                    attr_value(&mut attrs);
                    attr_default(&mut attrs);

                    attrs.push_str(" vals=");
                    attrs.push_str(&validator.values.to_json5().unwrap_or_default());

                    attrs
                }

                _ => {
                    // TODO: Implement encoding for other validators
                    losses.add("Parameter.validator");

                    String::new()
                }
            };

            md.push('{');
            md.push_str(&attrs);
            md.push('}');
        }

        // TODO other losses for executable nodes

        (md, losses)
    }
}
