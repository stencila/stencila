use serde_json::{Map, Value};
use stencila_codec::{
    Losses,
    eyre::{Result, bail},
    stencila_schema::{Article, Inline, Node, Text},
};

use crate::{
    blocks::{decode_block, encode_block},
    inlines::{decode_inline, encode_inline_children},
};

pub fn encode_document(node: &Node, losses: &mut Losses) -> Result<Value> {
    let Node::Article(article) = node else {
        bail!("OXA codec only supports encoding Article nodes");
    };

    let children: Vec<Value> = article
        .content
        .iter()
        .map(|b| encode_block(b, losses))
        .collect();

    let mut doc = Map::new();
    doc.insert("type".into(), Value::String("Document".into()));

    if let Some(title) = &article.title {
        doc.insert("title".into(), encode_inline_children(title, losses));
    }

    if let Some(metadata) = encode_metadata(article)? {
        doc.insert("metadata".into(), metadata);
    }

    doc.insert("children".into(), Value::Array(children));

    Ok(Value::Object(doc))
}

fn encode_metadata(article: &Article) -> Result<Option<Value>> {
    let Value::Object(mut metadata) = serde_json::to_value(article)? else {
        return Ok(None);
    };

    // These properties have dedicated OXA Document fields (or are its type
    // discriminator); everything else belongs in the arbitrary metadata object.
    for property in ["type", "title", "content"] {
        metadata.remove(property);
    }

    Ok((!metadata.is_empty()).then_some(Value::Object(metadata)))
}

pub fn decode_document(obj: &Map<String, Value>, losses: &mut Losses) -> Result<Node> {
    let content = obj
        .get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_object())
                .map(|o| decode_block(o, losses))
                .collect::<Result<Vec<_>>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;

    let mut article = decode_metadata(obj)?;
    article.content = content;

    decode_title(obj, &mut article, losses)?;

    Ok(Node::Article(article))
}

fn decode_title(
    obj: &Map<String, Value>,
    article: &mut Article,
    losses: &mut Losses,
) -> Result<()> {
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
                .map(|o| decode_inline(o, losses))
                .collect::<Result<Vec<_>>>()?;
            if !inlines.is_empty() {
                article.title = Some(inlines);
            }
        }
        _ => {}
    }

    Ok(())
}

fn decode_metadata(obj: &Map<String, Value>) -> Result<Article> {
    let mut article = obj
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    // Supply the structural properties required to deserialize an Article,
    // while preventing arbitrary OXA metadata from overriding them.
    article.insert("type".into(), Value::String("Article".into()));
    article.insert("content".into(), Value::Array(Vec::new()));
    article.remove("title");

    Ok(serde_json::from_value(Value::Object(article))?)
}
