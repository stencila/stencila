use serde_json::{Map, Value, json};
use stencila_codec::{
    eyre::{Result, bail},
    stencila_schema::{Article, Inline, Node, Text},
};

use crate::{
    blocks::{decode_block, encode_block},
    inlines::{decode_inline, encode_inline_children},
};

pub fn encode_document(node: &Node) -> Result<Value> {
    let Node::Article(article) = node else {
        bail!("OXA codec only supports encoding Article nodes");
    };

    let children: Vec<Value> = article.content.iter().map(encode_block).collect();

    let mut doc = json!({
        "type": "Document",
        "children": children,
    });

    if let Some(title) = &article.title {
        doc["title"] = encode_inline_children(title);
    }

    Ok(doc)
}

pub fn decode_document(obj: &Map<String, Value>) -> Result<Node> {
    let content = obj
        .get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_object())
                .map(decode_block)
                .collect::<Result<Vec<_>>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;

    let mut article = Article {
        content,
        ..Default::default()
    };

    decode_title(obj, &mut article)?;
    decode_metadata(obj, &mut article);

    Ok(Node::Article(article))
}

fn decode_title(obj: &Map<String, Value>, article: &mut Article) -> Result<()> {
    let Some(title_value) = obj.get("title") else {
        return Ok(());
    };

    match title_value {
        Value::String(s) => {
            article.title = Some(vec![Inline::Text(Text::new(s.as_str().into()))]);
        }
        Value::Array(arr) => {
            let inlines: Vec<Inline> = arr
                .iter()
                .filter_map(|v| v.as_object())
                .map(decode_inline)
                .collect::<Result<Vec<_>>>()?;
            if !inlines.is_empty() {
                article.title = Some(inlines);
            }
        }
        _ => {}
    }

    Ok(())
}

fn decode_metadata(obj: &Map<String, Value>, article: &mut Article) {
    let Some(metadata) = obj.get("metadata").and_then(|v| v.as_object()) else {
        return;
    };

    if let Some(doi) = metadata.get("doi").and_then(|v| v.as_str()) {
        article.doi = Some(doi.to_string());
    }

    if let Some(keywords) = metadata.get("keywords").and_then(|v| v.as_array()) {
        let kws: Vec<String> = keywords
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if !kws.is_empty() {
            article.options.keywords = Some(kws);
        }
    }
}
