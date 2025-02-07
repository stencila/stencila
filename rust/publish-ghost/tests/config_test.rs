use document::{schema::Node, Document};

#[cfg(test)]
mod tests {
    use super::*;
    use common::tempfile::tempdir;
    use common::{eyre::Result, tokio};
    use document::schema::{ConfigPublishGhostState, ConfigPublishGhostType};

    #[tokio::test]
    async fn test_ghost_config_parsing() -> Result<()> {
        // Create temporary directory with test file
        let dir = tempdir()?;
        let file_path = dir.path().join("test.md");

        // Create test content with YAML front matter
        let content = r#"---
config:
  publish:
    ghost:
      type: page
      slug: short-name-for-post-7
      featured: true
      state: draft
      tags:
      - Documentation
      - '#docs'
      incorrect value: 
---"#;

        std::fs::write(&file_path, content)?;

        let doc = Document::open(&file_path).await?;

        let ghost_config = doc
            .inspect(|root| {
                if let Node::Article(article) = root {
                    article
                        .config
                        .as_ref()
                        .and_then(|config| config.publish.as_ref())
                        .and_then(|publish| publish.ghost.clone())
                }else{
                    None
                }
            })
            .await;

        assert_eq!(ghost_config.clone().unwrap().r#type, Some(ConfigPublishGhostType::Page));
        assert_eq!(ghost_config.clone().unwrap().slug, Some("short-name-for-post-7".to_string()));
        assert_eq!(ghost_config.clone().unwrap().featured, Some(true));
        assert_eq!(ghost_config.clone().unwrap().state, Some(ConfigPublishGhostState::Draft));
        assert_eq!(
            ghost_config.clone().unwrap().tags,
            Some(vec!["Documentation".to_string(), "#docs".to_string()])
        );


        Ok(())
    }
}
