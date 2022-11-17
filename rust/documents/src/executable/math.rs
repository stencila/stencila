use common::{async_trait::async_trait, eyre::Result};
use graph_triples::resources::ResourceDigest;
use math_utils::to_mathml;
use stencila_schema::{MathBlock, MathFragment};

use crate::executable::{CompileContext, Executable};

#[async_trait]
impl Executable for MathBlock {
    /// Compile a `MathBlock` node
    ///
    /// Transpiles `text` to `mathml` property based on `mathLanguage`. Stores the
    /// `compileDigest` to avoid unnecessary re-transpilation. There is no
    /// need to add a resource to the context since there are never any dependencies
    /// between this and any other node.
    async fn compile(&mut self, _context: &mut CompileContext) -> Result<()> {
        let _id = ensure_id!(self, "mb", context);

        let compile_digest = Some(Box::new(
            ResourceDigest::from_strings(
                &[self.text.as_str(), self.math_language.as_str()].concat(),
                None,
            )
            .to_cord(),
        ));
        if compile_digest != self.compile_digest {
            match to_mathml(&self.math_language, &self.text, true) {
                Ok(mathml) => {
                    self.mathml = Some(Box::new(mathml));
                    self.compile_digest = compile_digest;
                    self.errors = None;
                }
                Err(error) => self.errors = Some(vec![error.to_string()]),
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Executable for MathFragment {
    /// Compile a `MathFragment` node
    ///
    /// As for `MatchBlock`.
    async fn compile(&mut self, _context: &mut CompileContext) -> Result<()> {
        let _id = ensure_id!(self, "mf", context);

        let compile_digest = Some(Box::new(
            ResourceDigest::from_strings(
                &[self.text.as_str(), self.math_language.as_str()].concat(),
                None,
            )
            .to_cord(),
        ));
        if compile_digest != self.compile_digest {
            match to_mathml(&self.math_language, &self.text, false) {
                Ok(mathml) => {
                    self.mathml = Some(Box::new(mathml));
                    self.compile_digest = compile_digest;
                    self.errors = None;
                }
                Err(error) => self.errors = Some(vec![error.to_string()]),
            }
        }

        Ok(())
    }
}
