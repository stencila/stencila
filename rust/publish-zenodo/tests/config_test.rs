use document::{schema::Node, Document};

#[cfg(test)]
mod tests {
    use super::*;
    use codec::schema::ConfigPublishZenodoAccessRight;
    use common::eyre::bail;
    use common::tempfile::tempdir;
    use common::{eyre::Result, tokio};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_ghost_config_parsing() -> Result<()> {
        // Create temporary directory with test file
        let dir = tempdir()?;
        let file_path = dir.path().join("test.md");

        // Create test content with YAML front matter
        let content = r#"---
config:
  publish:
    zenodo:
      access-right: embargoed 
      notes: Some extra notes
      method: A paragraph describing the methodology of the study.
      embargoed: '2025-02-04'
---"#;

        std::fs::write(&file_path, content)?;

        let doc = Document::open(&file_path, None).await?;

        let Some(config) = doc
            .inspect(|root| {
                if let Node::Article(article) = root {
                    article
                        .config
                        .as_ref()
                        .and_then(|config| config.publish.as_ref())
                        .and_then(|publish| publish.zenodo.clone())
                } else {
                    None
                }
            })
            .await
        else {
            bail!("Expected some config")
        };

        assert_eq!(
            config.access_right,
            Some(ConfigPublishZenodoAccessRight::Embargoed)
        );
        assert_eq!(config.notes, Some("Some extra notes".to_string()));
        assert_eq!(
            config.method,
            Some("A paragraph describing the methodology of the study.".to_string())
        );
        assert_eq!(
            config.embargoed.map(|date| date.value),
            Some("2025-02-04".to_string())
        );

        Ok(())
    }
}
