use std::collections::BTreeMap;

use crate::GraphBuilder;

use super::{
    analyze::analyze_source,
    facts::CodeFacts,
    language::CodeLanguage,
    project::{
        CodeGraphMode, CodeGraphSource, DocumentSymbolDefinition, DocumentSymbolUsage,
        ResourceResolver, add_code_facts_to_graph, add_document_function_use,
        add_document_symbol_derivation, add_document_symbol_use,
    },
};

/// Document-level code fact collector.
///
/// Document chunks and expressions are analyzed as they are encountered during
/// traversal. Cross-unit dependencies are represented by the underlying
/// `Generated` and `UsedBy` symbol edges rather than by materialized direct
/// code-unit edges.
#[derive(Debug, Default)]
pub(crate) struct DocumentCodeIndex {
    units: Vec<DocumentCodeUnit>,
}

#[derive(Debug)]
struct DocumentCodeUnit {
    unit_id: String,
    code: String,
    language: CodeLanguage,
    facts: CodeFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentDefinitionKind {
    Variable,
    Function,
}

#[derive(Debug, Clone, Copy)]
struct DocumentDefinition {
    unit_index: usize,
    kind: DocumentDefinitionKind,
}

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
                mode: CodeGraphMode::Document,
            },
            &facts,
            resolver,
        );
        self.units.push(DocumentCodeUnit {
            unit_id: unit_id.to_string(),
            code: code.to_string(),
            language,
            facts,
        });
    }

    /// Add dependencies between document code units after document-order
    /// definitions are known.
    pub(crate) fn finish(&self, builder: &mut GraphBuilder, scope: &str) {
        let mut definitions = BTreeMap::<(CodeLanguage, String), DocumentDefinition>::new();

        for (unit_index, unit) in self.units.iter().enumerate() {
            for flow in &unit.facts.variable_flows {
                if !unit.facts.assignments.contains(&flow.target)
                    || local_definition_available_at(&unit.facts, &flow.source, flow.target_offset)
                {
                    continue;
                }

                let Some(prior_definition) = definitions.get(&(unit.language, flow.source.clone()))
                else {
                    continue;
                };
                if prior_definition.kind != DocumentDefinitionKind::Variable {
                    continue;
                }
                let definition = &self.units[prior_definition.unit_index];
                add_document_symbol_derivation(
                    builder,
                    symbol_definition(scope, definition, &flow.source),
                    DocumentSymbolUsage {
                        scope,
                        unit_id: &unit.unit_id,
                        source_text: Some(&unit.code),
                        language: unit.language,
                        symbol: &flow.target,
                        offset: Some(flow.target_offset),
                    },
                );
            }

            for symbol in &unit.facts.uses {
                let Some(prior_definition) = definitions.get(&(unit.language, symbol.clone()))
                else {
                    continue;
                };
                let definition_unit = &self.units[prior_definition.unit_index];
                let definition = symbol_definition(scope, definition_unit, symbol);
                let usage = DocumentSymbolUsage {
                    scope,
                    unit_id: &unit.unit_id,
                    source_text: Some(&unit.code),
                    language: unit.language,
                    symbol,
                    offset: unit.facts.use_offsets.get(symbol).copied(),
                };
                match prior_definition.kind {
                    DocumentDefinitionKind::Variable => {
                        add_document_symbol_use(builder, definition, usage);
                    }
                    DocumentDefinitionKind::Function => {
                        add_document_function_use(builder, definition, usage);
                    }
                }
            }

            for symbol in &unit.facts.assignments {
                definitions.insert(
                    (unit.language, symbol.clone()),
                    DocumentDefinition {
                        unit_index,
                        kind: DocumentDefinitionKind::Variable,
                    },
                );
            }
            for symbol in &unit.facts.declarations {
                definitions.insert(
                    (unit.language, symbol.clone()),
                    DocumentDefinition {
                        unit_index,
                        kind: DocumentDefinitionKind::Function,
                    },
                );
            }
        }
    }
}

fn local_definition_available_at(facts: &CodeFacts, symbol: &str, offset: usize) -> bool {
    facts
        .definition_offsets
        .get(symbol)
        .is_some_and(|definition_offset| *definition_offset <= offset)
}

fn symbol_definition<'a>(
    scope: &'a str,
    unit: &'a DocumentCodeUnit,
    symbol: &'a str,
) -> DocumentSymbolDefinition<'a> {
    DocumentSymbolDefinition {
        scope,
        unit_id: &unit.unit_id,
        source_text: Some(&unit.code),
        language: unit.language,
        symbol,
        offset: unit.facts.definition_offsets.get(symbol).copied(),
    }
}
