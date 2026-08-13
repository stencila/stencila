//! JATS encoding shared by `MathInline` and `MathBlock`

use crate::prelude::*;

/// Whether a math language is TeX, which JATS writes as `<tex-math>`
fn is_tex(language: &str) -> bool {
    matches!(
        language.trim().to_lowercase().as_str(),
        "tex" | "latex" | "texmath" | "tex-math"
    )
}

/// Whether a math language is MathML, which JATS writes as `<mml:math>`
fn is_mathml(language: &str) -> bool {
    matches!(language.trim().to_lowercase().as_str(), "mathml" | "mml")
}

/// Whether the math of a formula is fully recoverable from the JATS elements
/// that it is emitted as
///
/// It is not for a language that JATS has no element for, such as AsciiMath, so
/// the caller keeps the source in attributes of Stencila's own instead.
pub(super) fn encodes_as_jats_math(math_language: Option<&str>, code: &str) -> bool {
    let language = math_language.map(str::trim).unwrap_or_default();
    code.trim().is_empty() || is_tex(language) || is_mathml(language)
}

/// Emit the content of a `<disp-formula>` or `<inline-formula>`
///
/// JATS states math as MathML, as TeX, or as both within `<alternatives>`, so
/// that is how it is written here rather than putting the source in an attribute
/// that only Stencila can read.
pub(super) fn encode_jats_math(
    code: &str,
    math_language: Option<&str>,
    mathml: Option<&str>,
    context: &mut JatsEncodeContext,
) {
    let code = code.trim();
    let language = math_language.map(str::trim).unwrap_or_default();

    // MathML compiled from the code is an alternative representation of it, not
    // a replacement, so it is only the whole of the math when the code is
    // MathML itself
    let code_mathml = is_mathml(language)
        .then_some(code)
        .filter(|code| !code.is_empty());
    let mathml = code_mathml.or_else(|| mathml.map(str::trim).filter(|mathml| !mathml.is_empty()));
    let tex = (is_tex(language) && !code.is_empty()).then_some(code);

    let alternatives = mathml.is_some() && tex.is_some();
    if alternatives {
        context.enter_elem("alternatives");
    }
    if let Some(tex) = tex {
        context.enter_elem("tex-math").push_text(tex).exit_elem();
    }
    if let Some(mathml) = mathml {
        // Already an `<mml:math>` element, or another element in the MathML
        // namespace, so is written as it is rather than wrapped again
        if mathml.starts_with('<') {
            context.push_xml(mathml);
        } else {
            context.enter_elem("mml:math").push_xml(mathml).exit_elem();
        }
    }
    if alternatives {
        context.exit_elem();
    }
}
