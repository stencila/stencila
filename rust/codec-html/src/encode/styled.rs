use codec::EncodeMode;
use hash_utils::str_seahash;
use stencila_schema::{Division, Span};

use crate::{EncodeContext, ToHtml};

use super::{attr, attr_id, attr_prop, attr_slot, elem, elem_placeholder, nothing};

/// Generate a `class` HTML attribute based on a class list and generated CSS
#[allow(clippy::box_collection)]
fn attr_class(
    classes: &Option<Vec<String>>,
    css: &Option<Box<String>>,
    context: &mut EncodeContext,
) -> String {
    let mut class = if let Some(css) = css {
        // Calculate a class name based on the digest of the CSS
        let digest = str_seahash(css).unwrap_or_default();
        let class = format!("st-{:x}", digest);

        // Add the CSS to the context for rendering to a <style> tag
        let css = css.replace(":root", &[".", &class].concat());
        context.styles.entry(class.clone()).or_insert(css);

        class
    } else {
        String::new()
    };

    if let Some(classes) = classes {
        if !classes.is_empty() {
            if !class.is_empty() {
                class.push(' ');
            }
            class.push_str(&classes.join(" "))
        }
    }

    if !class.is_empty() {
        attr("class", &class)
    } else {
        nothing()
    }
}

/// Escape CSS
fn escape_css(css: &String, context: &mut EncodeContext) -> String {
    if css.is_empty() {
        // It seems necessary for the CSS to have at least some content so that
        // the browser's mutation observer is able to observe the initial transpile of CSS.
        // Without this space, the first patch adding CSS, is ignored.
        " ".to_string()
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

        let (code, errors) = if context.options.mode >= EncodeMode::Inspect {
            (
                elem("pre", &[attr_slot("code")], &self.code),
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
            if let Some(css) = &self.css {
                elem(
                    "pre",
                    &[attr_prop("css"), attr_slot("css")],
                    &escape_css(css, context),
                )
            } else {
                nothing()
            }
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
            &[code, errors, css, content].concat(),
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

        let (code, errors) = if context.options.mode >= EncodeMode::Inspect {
            (
                elem("code", &[attr_slot("code")], &self.code),
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
            if let Some(css) = &self.css {
                elem(
                    "pre",
                    &[attr_prop("css"), attr_slot("css")],
                    &escape_css(css, context),
                )
            } else {
                nothing()
            }
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
            &[code, errors, css, content].concat(),
        )
    }
}
