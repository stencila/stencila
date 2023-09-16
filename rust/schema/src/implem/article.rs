use common::serde_yaml;

use crate::{prelude::*, Article};

impl Article {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = String::new();

        let mut yaml = serde_yaml::to_value(Self {
            // Avoid serializing content
            content: Vec::new(),
            ..self.clone()
        })
        .unwrap_or_default();

        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the type and (empty array) content
            yaml.remove("type");
            yaml.remove("content");

            // Only add a YAML header if there are remaining keys
            if !yaml.is_empty() {
                let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();
                md += "---\n";
                md += &yaml;
                md += "---\n\n";
            }
        }

        let (content_md, losses) = self.content.to_markdown();
        md += &content_md;

        (md, losses)
    }
}
