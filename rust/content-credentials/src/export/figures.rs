//! Parent-figure component aggregation for export credentials.

use std::{collections::BTreeMap, path::Path};

use stencila_codec_text_trait::to_text;
use stencila_schema::{Article, Block, Figure, Node};
use tempfile::{TempDir, tempdir};
use tokio::fs::write;

use crate::{
    CredentialProducer, CredentialProfile, IngredientRelationship, IngredientSnapshot, Result,
    SignAssetRequest, SourceRangeSnapshot, thumbnails,
};

use super::{
    components::ComponentIngredient,
    snapshot::{ExportSnapshotOptions, build_export_snapshot},
};

#[derive(Clone)]
struct FigureParent {
    node_id: String,
    node: Node,
    label: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Clone)]
struct FigureChildParent {
    parent: FigureParent,
    order: usize,
}

struct FigureGroup {
    parent: FigureParent,
    order: usize,
    children: Vec<(usize, ComponentIngredient)>,
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn group_figure_component_ingredients(
    producer: &CredentialProducer,
    node: &Node,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
    source_path: Option<&Path>,
    codec_name: &str,
    credential_profile: CredentialProfile,
    components: Vec<ComponentIngredient>,
    mut component_index: usize,
) -> Result<(Vec<ComponentIngredient>, Vec<TempDir>)> {
    let parents = figure_component_parents(node);
    if parents.is_empty() {
        return Ok((components, Vec::new()));
    }

    let mut direct = Vec::new();
    let mut grouped: BTreeMap<String, FigureGroup> = BTreeMap::new();

    for component in components {
        let child_parent = component
            .originating_id
            .as_deref()
            .and_then(|id| parents.get(id));

        if let Some(child_parent) = child_parent {
            let entry = grouped
                .entry(child_parent.parent.node_id.clone())
                .or_insert_with(|| FigureGroup {
                    parent: child_parent.parent.clone(),
                    order: child_parent.order,
                    children: Vec::new(),
                });
            entry.order = entry.order.min(child_parent.order);
            entry.children.push((child_parent.order, component));
        } else {
            direct.push(component);
        }
    }

    let mut temporary_dirs = Vec::new();
    let mut grouped = grouped.into_values().collect::<Vec<_>>();
    grouped.sort_by_key(|group| group.order);

    for group in grouped {
        if group.children.len() < 2 {
            direct.extend(
                group
                    .children
                    .into_iter()
                    .map(|(_order, component)| component),
            );
            continue;
        }

        let mut ordered_children = group.children;
        ordered_children.sort_by_key(|(order, _component)| *order);
        let children = ordered_children
            .into_iter()
            .map(|(_order, component)| component)
            .collect();

        let (component, temp_dir) = parent_figure_component_ingredient(
            producer,
            node,
            source_ranges,
            source_path,
            codec_name,
            credential_profile,
            component_index,
            group.parent,
            children,
        )
        .await?;
        component_index += 1;
        direct.push(component);
        temporary_dirs.push(temp_dir);
    }

    Ok((direct, temporary_dirs))
}

#[allow(clippy::too_many_arguments)]
async fn parent_figure_component_ingredient(
    producer: &CredentialProducer,
    root: &Node,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
    source_path: Option<&Path>,
    codec_name: &str,
    credential_profile: CredentialProfile,
    component_index: usize,
    parent: FigureParent,
    children: Vec<ComponentIngredient>,
) -> Result<(ComponentIngredient, TempDir)> {
    let temp_dir = tempdir()?;
    let asset_path = temp_dir
        .path()
        .join(format!("{}.svg", figure_asset_stem(&parent)));
    let component_labels = children
        .iter()
        .filter_map(|child| child.ingredient.title.clone())
        .collect::<Vec<_>>();
    write(&asset_path, figure_asset_svg(&parent, &component_labels)).await?;

    let mut provenance = build_export_snapshot(
        root,
        &parent.node,
        &asset_path,
        ExportSnapshotOptions {
            source_ranges,
            source_path,
            primary: false,
            asset_role: Some("figure"),
            asset_title: parent.title.as_deref(),
            asset_description: parent.description.as_deref(),
            codec_name: Some(codec_name),
            profile: credential_profile,
        },
    );
    provenance
        .ingredients
        .extend(children.into_iter().map(|child| child.ingredient));

    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            media_type: Some("image/svg+xml".to_string()),
            title: parent.title.clone(),
            credential_profile,
            provenance: Some(provenance),
            ..Default::default()
        })
        .await?;

    let manifest_source = signed
        .sidecar_path
        .clone()
        .unwrap_or_else(|| signed.asset_path.clone());
    let component = ComponentIngredient {
        ingredient: IngredientSnapshot {
            label: Some(format!("component-{component_index}")),
            title: parent.title,
            media_type: Some("image/svg+xml".to_string()),
            content_digest: Some(signed.signed_asset_digest),
            relationship: IngredientRelationship::ComponentOf,
            description: parent.description,
            manifest_source: Some(manifest_source),
            thumbnail: Some(thumbnails::ingredient_for_node_type("Figure")),
            ..Default::default()
        },
        originating_id: Some(parent.node_id),
    };

    Ok((component, temp_dir))
}

fn figure_component_parents(root: &Node) -> BTreeMap<String, FigureChildParent> {
    let mut parents = BTreeMap::new();
    let mut stack = Vec::new();
    let mut next_order = 0;
    walk_node_for_figure_parents(root, &mut stack, &mut parents, &mut next_order);
    parents
}

fn walk_node_for_figure_parents(
    node: &Node,
    stack: &mut Vec<FigureParent>,
    parents: &mut BTreeMap<String, FigureChildParent>,
    next_order: &mut usize,
) {
    match node {
        Node::Article(Article { content, .. }) => {
            walk_blocks_for_figure_parents(content, stack, parents, next_order);
        }
        Node::Figure(figure) => walk_figure_for_figure_parents(figure, stack, parents, next_order),
        _ => {}
    }
}

fn walk_blocks_for_figure_parents(
    blocks: &[Block],
    stack: &mut Vec<FigureParent>,
    parents: &mut BTreeMap<String, FigureChildParent>,
    next_order: &mut usize,
) {
    for block in blocks {
        match block {
            Block::Figure(figure) => {
                walk_figure_for_figure_parents(figure, stack, parents, next_order);
            }
            Block::CodeChunk(chunk) => {
                if let Some(parent) = stack.last() {
                    insert_child_parent(parents, chunk.node_id().to_string(), parent, next_order);
                }
            }
            Block::MathBlock(math) => {
                if let Some(parent) = stack.last() {
                    insert_child_parent(parents, math.node_id().to_string(), parent, next_order);
                }
            }
            Block::Table(table) => {
                if let Some(parent) = stack.last() {
                    insert_child_parent(parents, table.node_id().to_string(), parent, next_order);
                }
            }
            Block::AudioObject(_) | Block::ImageObject(_) | Block::VideoObject(_) => {
                if let Some(parent) = stack.last() {
                    insert_child_parent_if_absent(
                        parents,
                        parent.node_id.clone(),
                        parent,
                        next_order,
                    );
                }
            }
            _ => {}
        }
    }
}

fn walk_figure_for_figure_parents(
    figure: &Figure,
    stack: &mut Vec<FigureParent>,
    parents: &mut BTreeMap<String, FigureChildParent>,
    next_order: &mut usize,
) {
    let self_parent = figure_parent(figure);

    if let Some(parent) = stack.last() {
        insert_child_parent(parents, figure.node_id().to_string(), parent, next_order);
    }

    stack.push(self_parent);
    walk_blocks_for_figure_parents(&figure.content, stack, parents, next_order);
    stack.pop();
}

fn insert_child_parent(
    parents: &mut BTreeMap<String, FigureChildParent>,
    child_id: String,
    parent: &FigureParent,
    next_order: &mut usize,
) {
    let order = *next_order;
    *next_order += 1;
    parents.insert(
        child_id,
        FigureChildParent {
            parent: parent.clone(),
            order,
        },
    );
}

fn insert_child_parent_if_absent(
    parents: &mut BTreeMap<String, FigureChildParent>,
    child_id: String,
    parent: &FigureParent,
    next_order: &mut usize,
) {
    if parents.contains_key(&child_id) {
        *next_order += 1;
    } else {
        insert_child_parent(parents, child_id, parent, next_order);
    }
}

fn figure_parent(figure: &Figure) -> FigureParent {
    let metadata = figure_metadata(figure);
    FigureParent {
        node_id: figure.node_id().to_string(),
        node: Node::Figure(figure.clone()),
        label: figure.label.clone(),
        title: metadata.0,
        description: metadata.1,
    }
}

fn figure_metadata(figure: &Figure) -> (Option<String>, Option<String>) {
    let caption = figure.caption.as_deref().and_then(blocks_to_text);
    let prefix = figure
        .label
        .as_deref()
        .map(|label| format!("Figure {label}"));
    let description = match (prefix, caption) {
        (Some(prefix), Some(caption)) => Some(format!("{prefix}: {caption}")),
        (Some(prefix), None) => Some(prefix),
        (None, Some(caption)) => Some(caption),
        (None, None) => None,
    };
    let title = description.as_deref().map(first_sentence);
    (title, description)
}

fn blocks_to_text(blocks: &[Block]) -> Option<String> {
    let text = blocks
        .iter()
        .map(to_text)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(text)
}

fn first_sentence(text: &str) -> String {
    let text = text.trim();
    let Some(index) = text.find('.') else {
        return text.to_string();
    };
    text[..=index].trim().to_string()
}

fn figure_asset_stem(parent: &FigureParent) -> String {
    parent
        .title
        .as_deref()
        .or(parent.label.as_deref())
        .and_then(safe_file_stem)
        .unwrap_or_else(|| parent.node_id.clone())
}

fn figure_asset_svg(parent: &FigureParent, component_labels: &[String]) -> String {
    let title = parent
        .title
        .as_deref()
        .or(parent.description.as_deref())
        .unwrap_or("Figure");
    let subtitle = match component_labels {
        [] => "No components".to_string(),
        [one] => one.clone(),
        labels => format!("{} components", labels.len()),
    };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360" role="img" aria-label="{title}">
  <rect width="640" height="360" fill="#f8fafc"/>
  <rect x="28" y="28" width="584" height="304" rx="12" fill="#ffffff" stroke="#94a3b8" stroke-width="2"/>
  <text x="56" y="96" font-family="Arial, sans-serif" font-size="34" font-weight="700" fill="#0f172a">{title}</text>
  <text x="56" y="146" font-family="Arial, sans-serif" font-size="22" fill="#334155">{subtitle}</text>
  <g fill="#64748b">
    <rect x="56" y="208" width="148" height="84" rx="8"/>
    <rect x="246" y="208" width="148" height="84" rx="8"/>
    <rect x="436" y="208" width="148" height="84" rx="8"/>
  </g>
</svg>
"##,
        title = escape_xml(title),
        subtitle = escape_xml(&subtitle)
    )
}

fn escape_xml(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect(),
            '>' => "&gt;".chars().collect(),
            '"' => "&quot;".chars().collect(),
            '\'' => "&apos;".chars().collect(),
            _ => vec![character],
        })
        .collect()
}

fn safe_file_stem(value: &str) -> Option<String> {
    let mut stem = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            stem.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !stem.is_empty() {
            stem.push('-');
            previous_dash = true;
        }
    }

    if previous_dash {
        stem.pop();
    }

    (!stem.is_empty()).then_some(stem)
}

#[cfg(test)]
mod tests {
    use stencila_schema::{CodeChunk, Cord, ImageObject};

    use super::*;

    #[test]
    fn top_level_figure_maps_to_itself_for_direct_media_origins() {
        let figure = Figure {
            content: vec![
                Block::ImageObject(ImageObject::new("image.png".to_string())),
                Block::CodeChunk(CodeChunk::new(Cord::from("plot()"))),
            ],
            ..Default::default()
        };
        let figure_id = figure.node_id().to_string();
        let Block::CodeChunk(chunk) = &figure.content[1] else {
            unreachable!("test fixture should contain a code chunk")
        };
        let chunk_id = chunk.node_id().to_string();

        let parents = figure_component_parents(&Node::Figure(figure));

        assert_eq!(
            parents
                .get(&figure_id)
                .map(|child_parent| child_parent.parent.node_id.as_str()),
            Some(figure_id.as_str()),
            "direct media assets are attributed to the containing figure"
        );
        assert_eq!(
            parents
                .get(&chunk_id)
                .map(|child_parent| child_parent.parent.node_id.as_str()),
            Some(figure_id.as_str()),
            "code chunk outputs should still group under the containing figure"
        );
        assert!(
            parents[&figure_id].order < parents[&chunk_id].order,
            "component order should follow figure content order"
        );
    }

    #[test]
    fn nested_figure_still_maps_to_outer_parent() {
        let inner = Figure::new(Vec::new());
        let inner_id = inner.node_id().to_string();
        let outer = Figure::new(vec![Block::Figure(inner)]);
        let outer_id = outer.node_id().to_string();

        let parents = figure_component_parents(&Node::Figure(outer));

        assert_eq!(
            parents
                .get(&inner_id)
                .map(|child_parent| child_parent.parent.node_id.as_str()),
            Some(outer_id.as_str())
        );
    }
}
