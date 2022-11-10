use asciimath_rs::format::mathml::ToMathML;
use latex2mathml::{latex_to_mathml, DisplayStyle};

use common::eyre::Result;

/**
 * Transpile some text in a math language to MathML
 */
pub fn to_mathml(language: &str, text: &str, block: bool) -> Result<String> {
    Ok(match language {
        "asciimath" => {
            // A backslashes causes a panic within the `charred` crate
            // used by `asciimath_rs`. This removes all backslashes. The user can use
            // the "backslash" TeX alternative if they need a backslash in th math output.
            let text = text.replace('\\', "");
            format!(
                r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="{}">{}</math>"#,
                if block { "block" } else { "inline" },
                &asciimath_rs::parse(text).to_mathml()
            )
        }
        "mathml" => text.to_string(),
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
