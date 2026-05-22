use stencila_schema::{GraphEdgeKind, Node, SoftwareSourceCode};

use crate::{
    GraphBuilder, evidence,
    ids::{LocalGraphId, WorkspaceRelPath},
};

use super::{
    analyze::analyze_source,
    language::CodeLanguage,
    project::{ResourceResolver, UnitFacts, add_code_facts_to_graph},
    util::path_name,
};

/// Add static code graph facts for a workspace source file.
///
/// Workspace source files get their own `SoftwareSourceCode` node in addition
/// to the filesystem `File` node. The code node carries computational facts,
/// while the file node preserves inventory and filesystem provenance.
///
/// Static resource literals are resolved by the caller whenever they name
/// another workspace file or symbolic link. Unresolved literals remain scoped
/// synthetic resources so code facts are still visible for files outside the
/// workspace inventory.
pub(crate) fn add_workspace_code(
    builder: &mut GraphBuilder,
    rel: &WorkspaceRelPath,
    code: &str,
    language: CodeLanguage,
    mut resolver: impl FnMut(&str) -> Option<String>,
) -> UnitFacts {
    let unit_id = LocalGraphId::code_unit(rel.as_str());
    let mut node = SoftwareSourceCode::new(path_name(rel.as_str()), language.name().to_string());
    node.id = Some(unit_id.clone());
    node.path = Some(rel.as_str().to_string());
    builder.add_schema_node(unit_id.clone(), Node::SoftwareSourceCode(node));
    builder.add_edge_with_evidence(
        &unit_id,
        LocalGraphId::file(rel),
        GraphEdgeKind::PartOf,
        vec![evidence::static_analysis()],
    );

    let facts = analyze_source(language, code);
    let scope = rel.as_str();
    add_code_facts_to_graph(
        builder,
        &unit_id,
        scope,
        language,
        &facts,
        0,
        Some(&mut resolver as &mut ResourceResolver<'_>),
    )
}
