use hash_utils::str_seahash;
use stencila_schema::{Division, Span};

use crate::{EncodeContext, ToHtml};

use super::{attr, attr_id, attr_prop, attr_slot, elem, elem_placeholder};

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

impl ToHtml for Division {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let lang = attr("programming-language", &self.programming_language);

        let text = elem(
            "pre",
            &[attr_prop("text"), attr_slot("text")],
            &self.text.to_html(context),
        );

        let errors = elem_placeholder(
            "div",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        let css = elem("pre", &[attr_prop("css"), attr_slot("css")], &self.css);

        let content = elem(
            "div",
            &[
                attr_prop("content"),
                attr_slot("content"),
                attr_class(&self.classes, &self.css, context),
            ],
            &self.content.to_html(context),
        );

        elem(
            "stencila-division",
            &[attr_id(&self.id), lang],
            &[text, errors, css, content].concat(),
        )
    }
}

impl ToHtml for Span {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let lang = attr("programming-language", &self.programming_language);

        let text = elem(
            "pre",
            &[attr_prop("text"), attr_slot("text")],
            &self.text.to_html(context),
        );

        let errors = elem_placeholder(
            "span",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        let css = elem("pre", &[attr_prop("css"), attr_slot("css")], &self.css);

        let content = elem(
            "span",
            &[
                attr_prop("content"),
                attr_slot("content"),
                attr_class(&self.classes, &self.css, context),
            ],
            &self.content.to_html(context),
        );

        elem(
            "stencila-span",
            &[attr_id(&self.id), lang],
            &[text, errors, css, content].concat(),
        )
    }
}
