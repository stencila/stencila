use asciimath_rs::format::mathml::ToMathML;
use codec::common::eyre::Result;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use stencila_schema::{MathBlock, MathFragment};

use crate::{EncodeContext, ToHtml};

use super::{attr, attr_id, attr_itemprop, attr_itemtype, attr_slot, elem, elem_meta, nothing};

fn to_mathml(lang: Option<&String>, text: &str, block: bool) -> Result<String> {
    let mathml = match lang.map(|string| string.as_ref()) {
        Some("mathml") => text.to_string(),
        Some("asciimath") => format!(
            r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="{}">{}</math>"#,
            if block { "block" } else { "inline" },
            &asciimath_rs::parse(text).to_mathml()
        ),
        _ => latex_to_mathml(
            text,
            if block {
                DisplayStyle::Block
            } else {
                DisplayStyle::Inline
            },
        )?,
    };
    Ok(mathml)
}

impl ToHtml for MathBlock {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let (lang_attr, lang_meta) = match &self.math_language {
            Some(math_language) => (
                attr("math-language", math_language),
                elem_meta("mathLanguage", math_language),
            ),
            None => (nothing(), nothing()),
        };

        let text = elem(
            "code",
            &[
                attr_itemprop("text"),
                attr_slot("text"),
                attr("style", "display:none"),
            ],
            &self.text.to_html(context),
        );

        let math = match to_mathml(self.math_language.as_deref(), &self.text, true) {
            Ok(mathml) => mathml,
            Err(..) => elem("div", &[], &self.text),
        };

        elem(
            "stencila-math-block",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang_attr],
            &[lang_meta, text, math].concat(),
        )
    }
}

impl ToHtml for MathFragment {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let (lang_attr, lang_meta) = match &self.math_language {
            Some(math_language) => (
                attr("math-language", math_language),
                elem_meta("mathLanguage", math_language),
            ),
            None => (nothing(), nothing()),
        };

        let text = elem(
            "code",
            &[
                attr_itemprop("text"),
                attr_slot("text"),
                attr("style", "display:none"),
            ],
            &self.text.to_html(context),
        );

        let math = match to_mathml(self.math_language.as_deref(), &self.text, false) {
            Ok(mathml) => mathml,
            Err(..) => elem("span", &[], &self.text),
        };

        elem(
            "stencila-math-fragment",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang_attr],
            &[lang_meta, text, math].concat(),
        )
    }
}
