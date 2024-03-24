use schema::StyledInline;

use crate::prelude::*;

impl Executable for StyledInline {
    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Executing StyledInline {node_id}");

        let code = self.code.trim();
        if !code.is_empty() {
            let lang = self.style_language.as_deref().or_else(|| Some("style"));

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

        WalkControl::Break
    }
}
