//! Flatten a Stencila inline tree into AT Protocol richtext with facets.
//!
//! AT Protocol represents formatted text as a flat UTF-8 string annotated with
//! byte-range facets. This module walks the Stencila inline tree, accumulates
//! the plain text, and emits facets with multi-family features (OXA, Leaflet,
//! and optionally Bluesky) for each formatting span.

use serde_json::{Map, Value, json};
use stencila_codec::{
    Losses,
    stencila_schema::{
        CodeInline, Emphasis, Inline, Link, QuoteInline, Strikeout, Strong, Subscript, Superscript,
        Underline,
    },
};

use crate::nsids;

/// A byte range within a UTF-8 string.
#[derive(Debug)]
pub struct ByteSlice {
    /// The start byte offset (inclusive).
    pub byte_start: usize,
    /// The end byte offset (exclusive).
    pub byte_end: usize,
}

/// A typed feature carried by a facet.
#[derive(Debug)]
pub struct Feature {
    /// The NSID type string (e.g. `"pub.oxa.richtext.facet#emphasis"`).
    pub type_str: &'static str,
    /// Optional extra data (e.g. `uri` for link features).
    pub extra: Option<Map<String, Value>>,
}

/// A facet annotating a byte range of text with one or more features.
#[derive(Debug)]
pub struct Facet {
    /// The byte range this facet covers.
    pub index: ByteSlice,
    /// The features (from one or more families) for this byte range.
    pub features: Vec<Feature>,
}

/// Flattened richtext: a plain text string and its associated facets.
#[derive(Debug)]
pub struct RichText {
    /// The accumulated plain text.
    pub text: String,
    /// The facets annotating byte ranges within the text.
    pub facets: Vec<Facet>,
}

impl RichText {
    /// Serialize this richtext to a JSON value with `text` and optional `facets` fields.
    pub fn to_value(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("text".to_string(), Value::String(self.text.clone()));

        if !self.facets.is_empty() {
            let facets: Vec<Value> = self
                .facets
                .iter()
                .map(|facet| {
                    let features: Vec<Value> = facet
                        .features
                        .iter()
                        .map(|feature| {
                            let mut feat_obj = serde_json::Map::new();
                            feat_obj.insert(
                                "$type".to_string(),
                                Value::String(feature.type_str.to_string()),
                            );
                            if let Some(extra) = &feature.extra {
                                for (k, v) in extra {
                                    feat_obj.insert(k.clone(), v.clone());
                                }
                            }
                            Value::Object(feat_obj)
                        })
                        .collect();

                    json!({
                        "index": {
                            "byteStart": facet.index.byte_start,
                            "byteEnd": facet.index.byte_end,
                        },
                        "features": features,
                    })
                })
                .collect();
            obj.insert("facets".to_string(), Value::Array(facets));
        }

        Value::Object(obj)
    }
}

/// Flatten a slice of Stencila inlines into richtext with facets.
///
/// Walks the inline tree depth-first, accumulating plain text and emitting
/// facets with correct UTF-8 byte offsets for each formatting span.
/// Unsupported inline types have their text extracted and a loss recorded.
pub fn flatten_inlines(inlines: &[Inline], losses: &mut Losses) -> RichText {
    let mut text = String::new();
    let mut facets = Vec::new();
    flatten_inlines_inner(inlines, &mut text, &mut facets, losses);
    RichText { text, facets }
}

fn flatten_inlines_inner(
    inlines: &[Inline],
    text: &mut String,
    facets: &mut Vec<Facet>,
    losses: &mut Losses,
) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                text.push_str(&t.value);
            }
            Inline::Emphasis(Emphasis { content, .. }) => {
                flatten_with_features(
                    content,
                    &[(nsids::OXA_EMPHASIS, None), (nsids::LEAFLET_ITALIC, None)],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::Strong(Strong { content, .. }) => {
                flatten_with_features(
                    content,
                    &[(nsids::OXA_STRONG, None), (nsids::LEAFLET_BOLD, None)],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::CodeInline(CodeInline { code, .. }) => {
                let code_str = code.as_str();
                if !code_str.is_empty() {
                    let byte_start = text.len();
                    text.push_str(code_str);
                    let byte_end = text.len();
                    facets.push(Facet {
                        index: ByteSlice {
                            byte_start,
                            byte_end,
                        },
                        features: vec![
                            Feature {
                                type_str: nsids::OXA_INLINE_CODE,
                                extra: None,
                            },
                            Feature {
                                type_str: nsids::LEAFLET_CODE,
                                extra: None,
                            },
                        ],
                    });
                }
            }
            Inline::Subscript(Subscript { content, .. }) => {
                flatten_with_features(
                    content,
                    &[(nsids::OXA_SUBSCRIPT, None)],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::Superscript(Superscript { content, .. }) => {
                flatten_with_features(
                    content,
                    &[(nsids::OXA_SUPERSCRIPT, None)],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::Strikeout(Strikeout { content, .. }) => {
                flatten_with_features(
                    content,
                    &[
                        (nsids::OXA_STRIKETHROUGH, None),
                        (nsids::LEAFLET_STRIKETHROUGH, None),
                    ],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::Underline(Underline { content, .. }) => {
                flatten_with_features(
                    content,
                    &[
                        (nsids::OXA_UNDERLINE, None),
                        (nsids::LEAFLET_UNDERLINE, None),
                    ],
                    text,
                    facets,
                    losses,
                );
            }
            Inline::Link(Link {
                content, target, ..
            }) => {
                let uri_extra = {
                    let mut map = Map::new();
                    map.insert("uri".to_string(), Value::String(target.clone()));
                    map
                };

                let is_absolute = target.starts_with("https://") || target.starts_with("http://");

                let mut features: Vec<(&'static str, Option<Map<String, Value>>)> =
                    Vec::with_capacity(if is_absolute { 3 } else { 2 });
                features.push((nsids::OXA_LINK, Some(uri_extra.clone())));
                if is_absolute {
                    features.push((nsids::LEAFLET_LINK, Some(uri_extra.clone())));
                    features.push((nsids::BLUESKY_LINK, Some(uri_extra)));
                } else {
                    features.push((nsids::LEAFLET_LINK, Some(uri_extra)));
                }

                flatten_with_features(content, &features, text, facets, losses);
            }
            _ => {
                losses.add(format!("encode:AtProtoJson:{inline}"));
                flatten_inlines_inner_text(inline, text);
            }
        }
    }
}

fn flatten_with_features(
    content: &[Inline],
    features: &[(&'static str, Option<Map<String, Value>>)],
    text: &mut String,
    facets: &mut Vec<Facet>,
    losses: &mut Losses,
) {
    let byte_start = text.len();
    flatten_inlines_inner(content, text, facets, losses);
    let byte_end = text.len();

    if byte_end > byte_start {
        facets.push(Facet {
            index: ByteSlice {
                byte_start,
                byte_end,
            },
            features: features
                .iter()
                .map(|(type_str, extra)| Feature {
                    type_str,
                    extra: extra.clone(),
                })
                .collect(),
        });
    }
}

fn flatten_inlines_inner_text(inline: &Inline, text: &mut String) {
    match inline {
        Inline::Text(t) => text.push_str(&t.value),
        Inline::CodeInline(CodeInline { code, .. }) => {
            text.push_str(code.as_str());
        }
        Inline::Emphasis(Emphasis { content, .. })
        | Inline::Strong(Strong { content, .. })
        | Inline::Subscript(Subscript { content, .. })
        | Inline::Superscript(Superscript { content, .. })
        | Inline::Strikeout(Strikeout { content, .. })
        | Inline::Underline(Underline { content, .. })
        | Inline::Link(Link { content, .. })
        | Inline::QuoteInline(QuoteInline { content, .. }) => {
            for child in content {
                flatten_inlines_inner_text(child, text);
            }
        }
        _ => {}
    }
}
