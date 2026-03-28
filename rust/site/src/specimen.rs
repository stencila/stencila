use eyre::Result;
use stencila_codec::{DecodeOptions, stencila_schema::Node};
use stencila_codecs::from_str;
use stencila_format::Format;

pub const SPECIMEN_ROUTE: &str = "/_specimen/";

pub async fn specimen_node() -> Result<Node> {
    from_str(
        include_str!("specimen.md"),
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        }),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_codec::stencila_schema::{Article, Block};
    use stencila_node_stabilize::stabilize;

    async fn specimen_article() -> Article {
        let node = specimen_node()
            .await
            .expect("specimen_node() should decode successfully");

        match node {
            Node::Article(article) => article,
            other => panic!("specimen_node() should return an Article, got {other:?}"),
        }
    }

    fn has_block(content: &[Block], predicate: fn(&Block) -> bool) -> bool {
        content.iter().any(predicate)
    }

    #[tokio::test]
    async fn test_specimen_node_returns_article_with_expected_block_types() -> Result<()> {
        let article = specimen_article().await;

        assert!(
            has_block(&article.content, |b| matches!(b, Block::Heading(_))),
            "specimen Article should contain at least one Heading"
        );
        assert!(
            has_block(&article.content, |b| matches!(b, Block::Paragraph(_))),
            "specimen Article should contain at least one Paragraph"
        );
        assert!(
            has_block(&article.content, |b| matches!(b, Block::CodeBlock(_))),
            "specimen Article should contain at least one CodeBlock"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_specimen_node_covers_token_families() -> Result<()> {
        let article = specimen_article().await;
        let content = &article.content;

        // The specimen should contain block types representing the required
        // token families: Typography (Heading+Paragraph), Code (CodeBlock),
        // Lists (List), Blockquotes (QuoteBlock), Tables (Table),
        // Admonitions (Admonition), Figures (Figure), Math (MathBlock).

        assert!(
            has_block(content, |b| matches!(b, Block::List(_))),
            "specimen should contain at least one List"
        );
        assert!(
            has_block(content, |b| matches!(b, Block::QuoteBlock(_))),
            "specimen should contain at least one QuoteBlock"
        );
        assert!(
            has_block(content, |b| matches!(b, Block::Table(_))),
            "specimen should contain at least one Table"
        );
        assert!(
            has_block(content, |b| matches!(b, Block::Admonition(_))),
            "specimen should contain at least one Admonition"
        );
        assert!(
            has_block(content, |b| matches!(b, Block::Figure(_))),
            "specimen should contain at least one Figure"
        );
        assert!(
            has_block(content, |b| matches!(b, Block::MathBlock(_))),
            "specimen should contain at least one MathBlock"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_specimen_heading_slugs_after_stabilization() -> Result<()> {
        let mut node = specimen_node()
            .await
            .expect("specimen_node() should decode successfully");

        stabilize(&mut node);

        let Node::Article(article) = &node else {
            panic!("specimen_node() should return an Article");
        };

        let heading_ids: Vec<String> = article
            .content
            .iter()
            .filter_map(|block| {
                if let Block::Heading(heading) = block {
                    Some(heading.node_id().to_string())
                } else {
                    None
                }
            })
            .collect();

        let expected_slugs = [
            "hea_typography",
            "hea_code",
            "hea_lists",
            "hea_blockquotes",
            "hea_tables",
            "hea_admonitions",
            "hea_figures",
            "hea_math",
        ];

        for expected in &expected_slugs {
            assert!(
                heading_ids.iter().any(|id| id == expected),
                "Expected heading slug `{}` not found in heading IDs: {:?}",
                expected,
                heading_ids
            );
        }

        Ok(())
    }
}
