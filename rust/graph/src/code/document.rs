use std::collections::BTreeMap;

use stencila_schema::GraphEdgeKind;

use crate::{GraphBuilder, evidence};

use super::{
    analyze::analyze_source,
    language::CodeLanguage,
    project::{ResourceResolver, UnitFacts, add_code_facts_to_graph},
};

/// Document-level reactivity index.
///
/// Document chunks and expressions are analyzed as they are encountered during
/// traversal, but symbol dependencies can only be resolved after all earlier
/// and later units are known. This index stores compact per-unit summaries and
/// emits `DependsOn` edges during collector finalization.
#[derive(Debug, Default)]
pub(crate) struct DocumentCodeIndex {
    /// Code units discovered in document order.
    ///
    /// The order is needed to resolve a use to the closest prior definition,
    /// matching the way executable documents commonly build reactive state from
    /// top to bottom.
    units: Vec<UnitFacts>,
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
        let order = self.units.len();
        let facts = analyze_source(language, code);
        let summary =
            add_code_facts_to_graph(builder, unit_id, scope, language, &facts, order, resolver);
        self.units.push(summary);
    }

    /// Resolve symbol use/definition dependencies across document code units.
    ///
    /// Each unresolved use is matched to the nearest previous unit that defined
    /// the same symbol. The resulting edge points from dependent unit to source
    /// unit so document reactivity queries can ask what a chunk depends on.
    pub(crate) fn finish(&self, builder: &mut GraphBuilder) {
        let mut definitions = BTreeMap::<String, Vec<&UnitFacts>>::new();
        for unit in &self.units {
            for symbol in &unit.definitions {
                definitions.entry(symbol.clone()).or_default().push(unit);
            }
        }

        for unit in &self.units {
            for symbol in &unit.uses {
                let Some(source_unit) = definitions.get(symbol).and_then(|units| {
                    units
                        .iter()
                        .rev()
                        .find(|candidate| candidate.order < unit.order)
                        .copied()
                }) else {
                    continue;
                };

                builder.add_edge_with_evidence(
                    &unit.unit_id,
                    &source_unit.unit_id,
                    GraphEdgeKind::DependsOn,
                    vec![evidence::static_analysis()],
                );
            }
        }
    }
}
