use crate::GraphBuilder;

use super::{
    analyze::analyze_source,
    language::CodeLanguage,
    project::{CodeGraphMode, CodeGraphSource, ResourceResolver, add_code_facts_to_graph},
};

/// Document-level code fact collector.
///
/// Document chunks and expressions are analyzed as they are encountered during
/// traversal. Cross-unit dependencies are represented by the underlying
/// `Generated` and `UsedBy` symbol edges rather than by materialized direct
/// code-unit edges.
#[derive(Debug, Default)]
pub(crate) struct DocumentCodeIndex;

impl DocumentCodeIndex {
    /// Analyze a document code node and add its local facts to the graph.
    ///
    /// Local facts such as imports, calls, and generated variables can be
    /// projected immediately. Cross-unit dependencies are deferred by storing
    /// the returned summary until `finish` has a complete document-order view.
    pub(crate) fn add_unit(
        &mut self,
        builder: &mut GraphBuilder,
        scope: &str,
        unit_id: &str,
        code: &str,
        language: CodeLanguage,
        resolver: Option<&mut ResourceResolver<'_>>,
    ) {
        let facts = analyze_source(language, code);
        add_code_facts_to_graph(
            builder,
            CodeGraphSource {
                unit_id,
                scope,
                language,
                source_text: Some(code),
                mode: CodeGraphMode::Full,
            },
            &facts,
            resolver,
        );
    }
}
