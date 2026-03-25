use serde_json::Value;
use stencila_codec::{
    Losses,
    eyre::{Result, bail},
    stencila_schema::{Article, Node},
};

use crate::{blocks::encode_block, facets::flatten_inlines};

pub fn encode_article(node: &Node, losses: &mut Losses) -> Result<Value> {
    let article = match node {
        Node::Article(article) => article,
        _ => bail!("AT Protocol codec can only encode Article nodes"),
    };

    let mut obj = serde_json::Map::new();

    // Encode title
    if let Some(title_inlines) = &article.title {
        let rt = flatten_inlines(title_inlines, losses);
        let title_value = rt.to_value();
        obj.insert("title".to_string(), title_value);
    }

    // Encode blocks
    let blocks: Vec<Value> = article
        .content
        .iter()
        .filter_map(|block| encode_block(block, losses))
        .collect();
    obj.insert("blocks".to_string(), Value::Array(blocks));

    // Encode createdAt with date precedence
    let created_at = resolve_created_at(article);
    obj.insert("createdAt".to_string(), Value::String(created_at));

    // Record losses for dropped article properties
    if article.authors.is_some() {
        losses.add("encode:dropped_article_authors");
    }
    if article.r#abstract.is_some() {
        losses.add("encode:dropped_article_abstract");
    }

    Ok(Value::Object(obj))
}

fn resolve_created_at(article: &Article) -> String {
    article
        .date_published
        .as_ref()
        .or(article.options.date_created.as_ref())
        .or(article.options.date_modified.as_ref())
        .map(|date| format!("{}T00:00:00.000Z", date.value))
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
}
