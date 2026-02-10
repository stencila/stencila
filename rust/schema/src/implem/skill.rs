use crate::{Skill, prelude::*};

impl MarkdownCodec for Skill {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(raw_yaml) = &self.frontmatter
            && !raw_yaml.is_empty()
        {
            let yaml = nest_metadata(raw_yaml);
            context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                context.push_str("---\n");
                context.push_str(&yaml);
                context.push_str("\n---\n\n");
            });
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}

/// Nest non-spec frontmatter fields under `metadata:` per the Agent Skills Spec.
///
/// The spec defines `name`, `description`, `compatibility`, and `allowed-tools` as
/// top-level fields. All other fields (CreativeWork properties like `authors`, `version`,
/// `licenses`) are moved under a `metadata:` key.
fn nest_metadata(yaml: &str) -> String {
    let Ok(mut mapping) = serde_yaml::from_str::<serde_yaml::Mapping>(yaml) else {
        return yaml.to_string();
    };

    // Fields that belong at the top level per the Agent Skills Spec (plus `type` for Stencila)
    const SPEC_FIELDS: &[&str] = &[
        "type",
        "name",
        "description",
        "compatibility",
        "allowed-tools",
        "allowedTools",
        "allowed_tools",
        "metadata",
    ];

    // Collect non-spec field keys
    let non_spec_keys: Vec<serde_yaml::Value> = mapping
        .keys()
        .filter(|key| key.as_str().is_none_or(|s| !SPEC_FIELDS.contains(&s)))
        .cloned()
        .collect();

    if non_spec_keys.is_empty() {
        return yaml.to_string();
    }

    // Get or create the metadata mapping
    let mut meta = match mapping.remove("metadata") {
        Some(serde_yaml::Value::Mapping(m)) => m,
        _ => serde_yaml::Mapping::new(),
    };

    // Move non-spec fields into metadata (existing metadata entries take precedence)
    for key in non_spec_keys {
        if let Some(val) = mapping.remove(&key)
            && !meta.contains_key(&key) {
                meta.insert(key, val);
            }
    }

    if !meta.is_empty() {
        mapping.insert("metadata".into(), serde_yaml::Value::Mapping(meta));
    }

    serde_yaml::to_string(&mapping)
        .unwrap_or_else(|_| yaml.to_string())
        .trim()
        .to_string()
}
