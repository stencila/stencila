use schema::StyledInline;

use crate::prelude::*;

impl Executable for StyledInline {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        let compilation_digest = parsers::parse(
            &self.code,
            self.style_language.as_deref().unwrap_or_default(),
        )
        .compilation_digest;

        if Some(&compilation_digest) == self.options.compilation_digest.as_ref() {
            tracing::trace!("Skipping compiling StyledInline {node_id}");

            return WalkControl::Break;
        }

        tracing::trace!("Compiling StyledInline {node_id}");

        let code = self.code.trim();
        if !code.is_empty() {
            let lang = self.style_language.as_deref().or(Some("style"));

            let (result, messages) = executor
                .kernels()
                .await
                .execute(code, lang)
                .await
                .unwrap_or_else(|error| {
                    (
                        Vec::new(),
                        vec![error_to_message("While compiling style", error)],
                    )
                });

            let mut result = result.into_iter();
            let css = result.next();
            let class_list = result.next();

            let messages = (!messages.is_empty()).then_some(messages);

            executor.replace_properties(
                &node_id,
                [
                    (Property::Css, css.into()),
                    (Property::ClassList, class_list.into()),
                    (Property::CompilationMessages, messages.into()),
                ],
            );
        } else {
            executor.replace_properties(
                &node_id,
                [
                    (Property::Css, Value::None),
                    (Property::ClassList, Value::None),
                    (Property::CompilationMessages, Value::None),
                ],
            );
        };

        WalkControl::Continue
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        // Re-compile in case required variables were not available on compile
        // TODO: a more refined approached based on any interpolated dependencies is needed
        self.options.compilation_digest = None;
        self.compile(executor).await
    }
}
