use common::serde_yaml;

use crate::{prelude::*, Article};

impl Article {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut yaml = serde_yaml::to_value(Self {
            // Avoid serializing content
            content: Vec::new(),
            ..self.clone()
        })
        .unwrap_or_default();

        if let Some(yaml) = yaml.as_mapping_mut() {
            // Remove the (empty array) content
            yaml.remove("content");
        }

        let yaml = serde_yaml::to_string(&yaml).unwrap_or_default();

        let mut markdown = if yaml.is_empty() {
            String::new()
        } else {
            format!("---\n{yaml}---\n\n")
        };

        let (content_md, losses) = self.content.to_markdown();
        markdown += &content_md;

        (markdown, losses)
    }
}
