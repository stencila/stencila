//! Component ingredient helpers for export-time C2PA manifests.

use std::path::{Path, PathBuf};

use crate::{IngredientRelationship, IngredientSnapshot, thumbnails};

use super::ingredients::image_ingredient_thumbnail;

pub(super) struct ComponentIngredient {
    pub(super) ingredient: IngredientSnapshot,
    pub(super) originating_id: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn signed_component_ingredient(
    component_index: usize,
    originating_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    path: &Path,
    media_type: Option<String>,
    content_digest: String,
    manifest_source: PathBuf,
    node_type: Option<&str>,
) -> ComponentIngredient {
    let thumbnail = media_type
        .as_deref()
        .and_then(|media_type| image_ingredient_thumbnail(path, media_type))
        .or_else(|| node_type.map(thumbnails::ingredient_for_node_type));

    ComponentIngredient {
        ingredient: IngredientSnapshot {
            label: Some(format!("component-{component_index}")),
            title: title.or_else(|| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            }),
            media_type,
            content_digest: Some(content_digest),
            relationship: IngredientRelationship::ComponentOf,
            description,
            manifest_source: Some(manifest_source),
            thumbnail,
            ..Default::default()
        },
        originating_id,
    }
}
