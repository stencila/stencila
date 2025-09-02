//! Generation of reference documentation from Stencila Schema

use std::{
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use eyre::Result;
use glob::glob;
use inflector::Inflector;
use itertools::Itertools;
use strum::IntoEnumIterator;

use stencila_codecs::{CodecSupport, EncodeOptions, Format};
use stencila_schema::{Article, Inline, Node, NodeType, TableCell, shortcuts::*};

use crate::{
    schema::{Category, HtmlOptions, JatsOptions, MarkdownOptions, Schema},
    schemas::Schemas,
};

impl Schemas {
    /// Generate documentation for codecs
    ///
    /// Rather than create new documentation files for each codec
    /// this is designed to augment existing documentation. Add the
    /// following to a Markdown file in the `reference/formats`
    /// folder:
    ///
    /// <!-- prettier-ignore-start -->
    /// <!-- CODEC-DOCS:START -->
    ///
    /// <!-- CODEC-DOCS:STOP -->
    /// <!-- prettier-ignore-end -->
    #[allow(clippy::print_stderr)]
    pub async fn docs_codecs(&self) -> Result<()> {
        eprintln!("Generating documentation for codecs");

        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/formats");

        const START: &str = "<!-- CODEC-DOCS:START -->";
        const STOP: &str = "<!-- CODEC-DOCS:STOP -->";

        for file in glob(&dest.join("*.md").as_os_str().to_string_lossy())?.flatten() {
            let Some(name) = file.file_stem() else {
                continue;
            };
            let format = Format::from_name(name.to_string_lossy().as_ref());

            let Ok(codec) = stencila_codecs::get(None, Some(&format), None) else {
                continue;
            };

            let mut content = fs::read_to_string(&file)?;

            let start = content.find(START);
            let stop = content.rfind(STOP);

            let (Some(start), Some(stop)) = (start, stop) else {
                continue;
            };

            let mut items = Vec::new();
            if codec.supports_from_path() {
                items.push(li([t("decoding from a file")]))
            }
            if codec.supports_from_string() {
                items.push(li([t("decoding from a string")]))
            }
            if codec.supports_to_path() {
                items.push(li([t("encoding to a file")]))
            }
            if codec.supports_to_string() {
                items.push(li([t("encoding to a string")]))
            }

            let mut rows = vec![tr([
                th([t("Node type")]),
                th([t("Encoding")]),
                th([t("Decoding")]),
                th([t("Notes")]),
            ])];
            for category in Category::iter() {
                rows.push(tr([td([stg([t(category.to_string().to_title_case())])])]));

                for (title, schema) in self
                    .schemas
                    .iter()
                    .filter(|(_, schema)| schema.category == category)
                    .sorted_by(|(a, ..), (b, ..)| a.cmp(b))
                {
                    let Ok(node_type) = NodeType::from_str(title) else {
                        continue;
                    };

                    let title = td([lnk(
                        [t(title)],
                        format!(
                            "https://stencila.ghost.io/docs/reference/schema/{}",
                            title.to_snake_case()
                        ),
                    )]);

                    fn codec_support(support: CodecSupport) -> TableCell {
                        match support {
                            CodecSupport::None => td([]),
                            support => td([t(format!(
                                "{icon} {desc}",
                                icon = match support {
                                    CodecSupport::NoLoss => "🟢",
                                    CodecSupport::LowLoss => "🔷",
                                    CodecSupport::HighLoss => "⚠️",
                                    CodecSupport::None => "",
                                },
                                desc = support.to_string().to_sentence_case()
                            ))]),
                        }
                    }

                    let encoding = codec_support(codec.supports_to_type(node_type));
                    let decoding = codec_support(codec.supports_from_type(node_type));
                    let notes = td(Schemas::docs_format_notes(schema, format.clone()));

                    rows.push(tr([title, encoding, decoding, notes]));
                }
            }

            let article = Article {
                content: vec![
                    h1([t("Support")]),
                    p([t(format!(
                        "Stencila supports these operations for {}:",
                        format.name()
                    ))]),
                    ul(items),
                    p([t("Support and degree of loss by node type:")]),
                    tbl(rows),
                    p([
                        t("See the Rust crate "),
                        lnk(
                            [ci(format!("codec-{}", codec.name()))],
                            format!(
                                "https://github.com/stencila/stencila/tree/main/rust/codec-{}",
                                codec.name()
                            ),
                        ),
                        t(" for more details."),
                    ]),
                ],
                ..Default::default()
            };

            let md = stencila_codecs::to_string(
                &Node::Article(article),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await?;
            let md = ["\n\n", &md, "\n\n"].concat();

            content.replace_range(start.saturating_add(START.len())..stop, &md);

            fs::write(file, content)?;
        }

        Ok(())
    }

    /// Generates notes for a schema and format
    pub fn docs_format_notes(schema: &Schema, template: Format) -> Vec<Inline> {
        if let (Format::Html, Some(HtmlOptions { special, elem, .. })) = (&template, &schema.html) {
            if *special {
                if let Some(elem) = elem {
                    vec![
                        t("Encoded as "),
                        lnk(
                            [ci(format!("<{elem}>"))],
                            format!(
                                "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/{elem}"
                            ),
                        ),
                        t(" using special function"),
                    ]
                } else {
                    vec![t("Encoded using special function")]
                }
            } else if let Some(elem) = elem {
                vec![
                    t("Encoded as "),
                    lnk(
                        [ci(format!("<{elem}>"))],
                        format!("https://developer.mozilla.org/en-US/docs/Web/HTML/Element/{elem}"),
                    ),
                ]
            } else {
                vec![t("Encoded using derived function")]
            }
        } else if let (Format::Jats, Some(JatsOptions { elem, special, .. })) =
            (&template, &schema.jats)
        {
            if *special {
                if let Some(elem) = elem {
                    vec![
                        t("Encoded as "),
                        lnk(
                            [ci(format!("<{elem}>"))],
                            format!(
                                "https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/{elem}.html"
                            ),
                        ),
                        t(" using special function"),
                    ]
                } else {
                    vec![t("Encoded using special function")]
                }
            } else if let Some(elem) = elem {
                vec![
                    t("Encoded as "),
                    lnk(
                        [ci(format!("<{elem}>"))],
                        format!(
                            "https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/{elem}.html"
                        ),
                    ),
                ]
            } else {
                vec![t("Encoded using derived function")]
            }
        } else if let (
            Format::Markdown,
            Some(MarkdownOptions {
                derive, template, ..
            }),
        ) = (&template, &schema.markdown)
        {
            if !derive {
                vec![t("Encoded using implemented function")]
            } else if let Some(template) = template {
                vec![t("Encoded as "), ci(template)]
            } else {
                vec![t("Encoded using derived function")]
            }
        } else {
            vec![]
        }
    }
}
