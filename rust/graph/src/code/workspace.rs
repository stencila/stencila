use stencila_schema::{DateTime, Node, SoftwareSourceCode};

use crate::{GraphBuilder, evidence, ids::WorkspaceRelPath};

use super::{
    analyze::analyze_source,
    language::CodeLanguage,
    project::{ResourceResolver, add_code_facts_to_graph},
    util::path_name,
};

/// Source file metadata needed to add a workspace code node.
pub(crate) struct WorkspaceCode<'a> {
    pub(crate) unit_id: &'a str,
    pub(crate) rel: &'a WorkspaceRelPath,
    pub(crate) code: &'a str,
    pub(crate) language: CodeLanguage,
    pub(crate) parent_id: Option<String>,
    pub(crate) date_created: Option<DateTime>,
    pub(crate) date_modified: Option<DateTime>,
}

/// Add static code graph facts for a workspace source file.
///
/// Workspace source files are represented by a single `SoftwareSourceCode` node.
/// The code node carries both path identity and computational facts so focused
/// graph views do not need to reconcile separate file and code nodes for the
/// same source file.
///
/// Static resource literals are resolved by the caller whenever they name
/// another workspace file or symbolic link. Unresolved literals remain scoped
/// synthetic resources so code facts are still visible for files outside the
/// workspace inventory.
pub(crate) fn add_workspace_code(
    builder: &mut GraphBuilder,
    source: WorkspaceCode,
    mut resolver: impl FnMut(&str) -> Option<String>,
) {
    let mut node = SoftwareSourceCode::new(
        path_name(source.rel.as_str()),
        source.language.name().to_string(),
    );
    node.id = Some(source.unit_id.to_string());
    node.path = Some(source.rel.as_str().to_string());
    node.options.date_created = source.date_created;
    node.options.date_modified = source.date_modified;
    builder.add_schema_node(source.unit_id, Node::SoftwareSourceCode(node));
    if let Some(parent_id) = source.parent_id {
        builder.add_containment(source.unit_id, parent_id, vec![evidence::observed()]);
    }

    let facts = analyze_source(source.language, source.code);
    let scope = source.rel.as_str();
    add_code_facts_to_graph(
        builder,
        source.unit_id,
        scope,
        source.language,
        &facts,
        Some(&mut resolver as &mut ResourceResolver<'_>),
    );
}
