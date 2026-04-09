use std::{collections::HashMap, future::Future, path::PathBuf};

use eyre::Result;
use stencila_codec::{DecodeOptions, stencila_schema::Node};
use stencila_codecs::from_str;
use stencila_format::Format;

use crate::layout::render_layout;

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

/// Render the specimen page to the output directory
///
/// Writes the embedded specimen Markdown to a temp file and passes it
/// through the same `decode_document_fn` callback that `render()` uses for
/// regular documents. This means the specimen is *executed* — math is
/// typeset, diagrams are rendered, and plots are generated — rather than
/// merely decoded.
///
/// Layout resolution follows a three-tier fallback:
/// `site.specimen.layout` → `site.layout` → `LayoutPreset::Docs` defaults.
/// See the inline comment on `effective_layout` for rationale.
///
/// Returns the rendered HTML string.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn render_specimen_page<F, Fut>(
    site_config: &stencila_config::SiteConfig,
    base_url: &str,
    web_base: Option<&str>,
    output_dir: &std::path::Path,
    routes: &[crate::RouteEntry],
    routes_set: &std::collections::HashSet<String>,
    nav_items: &Vec<stencila_config::NavItem>,
    resolved_logo: Option<&stencila_config::LogoConfig>,
    workspace_id: Option<&str>,
    decode_document_fn: &F,
) -> Result<String>
where
    F: Fn(PathBuf, HashMap<String, String>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Node>> + Send,
{
    use stencila_codec::EncodeOptions;
    use stencila_node_stabilize::stabilize;
    use tokio::fs::{create_dir_all, write};

    // Write the embedded specimen Markdown to a temp file so it can be
    // passed through the decode_document_fn callback, which opens and
    // executes documents from disk (rendering math, diagrams, plots, etc.).
    let temp_dir = tempfile::tempdir()?;
    let specimen_path = temp_dir.path().join("specimen.md");
    tokio::fs::write(&specimen_path, include_str!("specimen.md")).await?;

    let mut node = decode_document_fn(specimen_path, HashMap::new()).await?;

    // Stabilize node UIDs for deterministic heading IDs
    stabilize(&mut node);

    // Resolve the effective layout for the specimen page using a three-tier
    // fallback chain:
    //
    // 1. `site.specimen.layout` — explicit specimen-specific override
    // 2. `site.layout` — the site's own layout, so the specimen mirrors
    //    what normal pages look like
    // 3. `LayoutPreset::Docs` defaults — a full layout with every region
    //    enabled (header, both sidebars, top, bottom, footer)
    //
    // The third tier matters because the specimen page is a diagnostic tool
    // primarily consumed by AI agents reviewing theme tokens and layout
    // components. When no layout is configured at all, falling through to
    // an empty `LayoutConfig::default()` would render a bare content-only
    // page with no regions to inspect, defeating the purpose. The Docs
    // preset is the richest built-in layout and ensures every region and
    // its typical components are visible for review.
    let effective_layout = site_config
        .specimen
        .as_ref()
        .and_then(|s| s.layout.clone())
        .or_else(|| site_config.layout.clone())
        .or_else(|| Some(stencila_config::LayoutPreset::Docs.defaults()));

    let specimen_site_config = stencila_config::SiteConfig {
        layout: effective_layout,
        ..site_config.clone()
    };

    // Render layout for the specimen route
    let breadcrumbs_map = HashMap::new();
    let layout_html = render_layout(
        &specimen_site_config,
        SPECIMEN_ROUTE,
        routes,
        routes_set,
        nav_items,
        &breadcrumbs_map,
        resolved_logo,
        workspace_id,
        None, // no git_repo_root
        None, // no git_origin
        None, // no git_branch
    );

    let site = format!("<body>\n{layout_html}\n</body>");

    let (html, ..) = stencila_codec_dom::encode(
        &node,
        Some(EncodeOptions {
            base_url: Some(base_url.to_string()),
            web_base: web_base.map(|s| s.to_string()),
            view: Some("site".to_string()),
            ..Default::default()
        }),
        Some(site),
    )
    .await?;

    // Write to _specimen/index.html
    let specimen_dir = output_dir.join("_specimen");
    create_dir_all(&specimen_dir).await?;
    let html_file = specimen_dir.join("index.html");
    write(&html_file, &html).await?;

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_codec::stencila_schema::{Article, Block};

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
        assert!(
            has_block(content, |b| matches!(b, Block::ThematicBreak(_))),
            "specimen should contain at least one ThematicBreak"
        );

        Ok(())
    }
}
