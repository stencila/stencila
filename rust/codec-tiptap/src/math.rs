//! Shared conversion helpers for native Tiptap math nodes.

use stencila_codec::stencila_schema::{CompilationMessage, ImageObject, MathBlock, MathInline};

use crate::{shared::TiptapDecodeContext, tiptap::MathAttrs};

/// Extract required math source code from native Tiptap attrs.
///
/// Stencila math nodes require a source string, but hand-edited or older Tiptap
/// JSON may omit it. Returning an empty string after recording a loss keeps
/// decoding total while making the data issue visible to callers.
pub(super) fn math_code_from_tiptap(
    attrs: &MathAttrs,
    prefix: &str,
    context: &mut TiptapDecodeContext,
) -> String {
    match attrs.code.clone() {
        Some(code) => code,
        None => {
            context.losses.add(format!("{prefix}.code"));
            String::new()
        }
    }
}

/// Build shared native Tiptap attrs for a Stencila math block.
pub(super) fn math_attrs_from_block(math_block: &MathBlock) -> MathAttrs {
    math_attrs_to_tiptap(
        math_block.id.clone(),
        math_block.code.to_string(),
        math_block.math_language.clone(),
        math_block.options.compilation_messages.clone(),
        math_block.options.mathml.clone(),
        math_block.options.images.clone(),
    )
}

/// Build shared native Tiptap attrs for a Stencila math inline.
pub(super) fn math_attrs_from_inline(math_inline: &MathInline) -> MathAttrs {
    math_attrs_to_tiptap(
        math_inline.id.clone(),
        math_inline.code.to_string(),
        math_inline.math_language.clone(),
        math_inline.options.compilation_messages.clone(),
        math_inline.options.mathml.clone(),
        math_inline.options.images.clone(),
    )
}

fn math_attrs_to_tiptap(
    id: Option<String>,
    code: String,
    math_language: Option<String>,
    compilation_messages: Option<Vec<CompilationMessage>>,
    mathml: Option<String>,
    images: Option<Vec<ImageObject>>,
) -> MathAttrs {
    MathAttrs {
        id,
        code: Some(code),
        math_language,
        compilation_messages,
        mathml,
        images,
    }
}
