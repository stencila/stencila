use codec::EncodeMode;
use stencila_schema::{MathBlock, MathFragment};

use crate::{EncodeContext, ToHtml};

use super::{attr_and_meta, attr_id, attr_itemprop, attr_itemtype, attr_slot, elem, nothing};

impl ToHtml for MathBlock {
    /// Encode a `MathBlock` to HTML
    ///
    /// If the mode is `Dynamic` or higher then encodes the MathML as a raw
    /// string that can be patched (and then rendered using a Web Component)
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let lang = attr_and_meta("math_language", &self.math_language);

        let text = if context.options.mode >= EncodeMode::Inspect {
            elem(
                "pre",
                &[attr_itemprop("text"), attr_slot("text")],
                &self.text.to_html(context),
            )
        } else {
            nothing()
        };

        let mathml = self.mathml.as_deref().unwrap_or(&self.text);

        let math = elem(
            "pre",
            &[attr_slot("mathml")],
            &if context.options.mode == EncodeMode::Static {
                mathml.to_string()
            } else {
                mathml.to_html(context) // Escape to a string
            },
        );

        elem(
            "stencila-math-block",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang.0],
            &[lang.1, text, math].concat(),
        )
    }
}

impl ToHtml for MathFragment {
    /// Encode a `MathFragment` to HTML
    ///
    /// As for `MathBlock`.
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let lang = attr_and_meta("math_language", &self.math_language);

        let text = if context.options.mode >= EncodeMode::Inspect {
            elem(
                "code",
                &[attr_itemprop("text"), attr_slot("text")],
                &self.text.to_html(context),
            )
        } else {
            nothing()
        };

        let mathml = self.mathml.as_deref().unwrap_or(&self.text);

        let math = elem(
            "span",
            &[attr_slot("mathml")],
            &if context.options.mode == EncodeMode::Static {
                mathml.to_string()
            } else {
                mathml.to_html(context) // Escape to a string
            },
        );

        elem(
            "stencila-math-fragment",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang.0],
            &[lang.1, text, math].concat(),
        )
    }
}
