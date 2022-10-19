use asciimath_rs::format::mathml::ToMathML;
use latex2mathml::{latex_to_mathml, DisplayStyle};

use common::eyre::Result;

/**
 * Transpile some text in a math language to MathML
 */
pub fn to_mathml(language: &str, text: &str, block: bool) -> Result<String> {
    Ok(match language {
        "mathml" => text.to_string(),
        "asciimath" => format!(
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
    })
}
