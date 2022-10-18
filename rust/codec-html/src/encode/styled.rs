use codec::EncodeMode;
use hash_utils::str_seahash;
use stencila_schema::{Division, Span};

use crate::{EncodeContext, ToHtml};

use super::{attr, attr_id, attr_prop, attr_slot, elem, elem_placeholder, nothing};

/// Generate a `class` HTML attribute based on a class list and generated CSS
fn attr_class(classes: &[String], css: &str, context: &mut EncodeContext) -> String {
    // Calculate a class name based on the digest of the CSS
    let digest = str_seahash(css).unwrap_or_default();
    let class_name = format!("st-{:x}", digest);

    // Add the CSS to the context for rendering to a <style> tag
    let css = css.replace(":root", &[".", &class_name].concat());
    context.styles.entry(class_name.clone()).or_insert(css);

    // Return the attribute
    let mut class = class_name;
    if !classes.is_empty() {
        class.push(' ');
        class.push_str(&classes.join(" "))
    }
    attr("class", &class)
}

/// Escape CSS
fn escape_css(css: &String, context: &mut EncodeContext) -> String {
    if css.is_empty() {
        // It seems necessary for the CSS to have at least some content so that
        // the browser's mutation observer is able to observe the initial transpile of CSS.
        // Without it, the first patch adding CSS is ignored.
        "\n/**/".to_string()
    } else {
        // HTML escape the CSS
        css.to_html(context)
    }
}

impl ToHtml for Division {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let attrs = if context.options.mode >= EncodeMode::Inspect {
            let programming_language = attr("programming-language", &self.programming_language);

            let guess_language = match self.guess_language {
                Some(value) => attr("guess-language", &value.to_string()),
                _ => nothing(),
            };

            vec![attr_id(&self.id), programming_language, guess_language]
        } else if context.options.mode >= EncodeMode::Dynamic {
            vec![attr_id(&self.id)]
        } else {
            vec![]
        };

        let (text, errors) = if context.options.mode >= EncodeMode::Inspect {
            (
                elem("pre", &[attr_slot("text")], &self.text),
                elem_placeholder(
                    "div",
                    &[attr_prop("errors"), attr_slot("errors")],
                    &self.errors,
                    context,
                ),
            )
        } else {
            (nothing(), nothing())
        };

        let css = if context.options.mode >= EncodeMode::Dynamic {
            elem(
                "pre",
                &[attr_prop("css"), attr_slot("css")],
                &escape_css(&self.css, context),
            )
        } else {
            nothing()
        };

        let content = elem(
            "div",
            &[
                attr_slot("content"),
                attr_class(&self.classes, &self.css, context),
            ],
            &self.content.to_html(context),
        );

        elem(
            "stencila-division",
            &attrs,
            &[text, errors, css, content].concat(),
        )
    }
}

impl ToHtml for Span {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let attrs = if context.options.mode >= EncodeMode::Inspect {
            let programming_language = attr("programming-language", &self.programming_language);

            let guess_language = match self.guess_language {
                Some(value) => attr("guess-language", &value.to_string()),
                _ => nothing(),
            };

            vec![attr_id(&self.id), programming_language, guess_language]
        } else if context.options.mode >= EncodeMode::Dynamic {
            vec![attr_id(&self.id)]
        } else {
            vec![]
        };

        let (text, errors) = if context.options.mode >= EncodeMode::Inspect {
            (
                elem("code", &[attr_slot("text")], &self.text),
                elem_placeholder(
                    "span",
                    &[attr_prop("errors"), attr_slot("errors")],
                    &self.errors,
                    context,
                ),
            )
        } else {
            (nothing(), nothing())
        };

        let css = if context.options.mode >= EncodeMode::Dynamic {
            elem(
                "code",
                &[attr_prop("css"), attr_slot("css")],
                &escape_css(&self.css, context),
            )
        } else {
            nothing()
        };

        let content = elem(
            "span",
            &[
                attr_slot("content"),
                attr_class(&self.classes, &self.css, context),
            ],
            &self.content.to_html(context),
        );

        elem(
            "stencila-span",
            &attrs,
            &[text, errors, css, content].concat(),
        )
    }
}
