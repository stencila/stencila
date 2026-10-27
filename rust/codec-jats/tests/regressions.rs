use stencila_codec::{
    DecodeInfo, EncodeOptions,
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Article, Author, AuthorRoleAuthor, AuthorRoleName, Block, CreativeWorkType,
        CreativeWorkVariant, Inline, IntegerOrString, Node, PersonOrOrganization, Primitive,
        SectionType,
    },
};
use stencila_codec_jats::{decode, encode};
use stencila_codec_text_trait::to_text;

fn article(source: &str) -> Result<(Article, DecodeInfo)> {
    let (node, info) = decode(source, None)?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    Ok((article, info))
}

fn assert_roundtrip(article: &Article) -> Result<()> {
    let expected = Node::Article(article.clone());
    for compact in [true, false] {
        let (encoded, _) = encode(
            &expected,
            Some(EncodeOptions {
                compact: Some(compact),
                ..Default::default()
            }),
        )?;
        let (actual, _) = decode(&encoded, None)?;
        assert_eq!(actual, expected, "compact={compact}: {encoded}");
    }
    Ok(())
}

#[test]
fn reference_translated_abstracts() -> Result<()> {
    let primary = "<abstract><p>Resumo original.</p></abstract>";
    let translation = r#"<trans-abstract id="en" xml:lang="EN">
        <p>English <italic>translation</italic>.</p>
      </trans-abstract>"#;
    // A translation must remain a part even before, or without, a primary abstract.
    for abstracts in [
        format!("{primary}{translation}"),
        format!("{translation}{primary}"),
        translation.to_string(),
    ] {
        let (article, info) = article(&format!(
            "<article><front><article-meta>{abstracts}</article-meta></front></article>"
        ))?;
        assert_eq!(
            to_text(&article.r#abstract).trim(),
            if abstracts.contains(primary) {
                "Resumo original."
            } else {
                ""
            }
        );
        let parts = article
            .options
            .parts
            .as_ref()
            .ok_or_eyre("missing translation")?;
        assert_eq!(parts.len(), 1);
        let CreativeWorkVariant::Article(part) = &parts[0] else {
            bail!("expected an article part")
        };
        assert_eq!(part.id.as_deref(), Some("en"));
        assert_eq!(to_text(&part.r#abstract).trim(), "English translation.");
        let Some(Block::Paragraph(paragraph)) =
            part.r#abstract.as_ref().and_then(|blocks| blocks.first())
        else {
            bail!("expected a translated paragraph")
        };
        assert!(matches!(
            paragraph.content.get(1),
            Some(Inline::Emphasis(..))
        ));
        assert_eq!(
            part.options
                .extra
                .as_ref()
                .and_then(|extra| extra.get("inLanguage")),
            Some(&Primitive::String("en".into()))
        );
        assert!(
            info.losses.is_empty(),
            "{:?}",
            info.losses.iter().collect::<Vec<_>>()
        );
        let (_, info) = encode(&Node::Article(article.clone()), None)?;
        assert!(
            info.losses.is_empty(),
            "translation should encode without losses"
        );
        assert_roundtrip(&article)?;
    }
    Ok(())
}

#[test]
fn reference_abstract_languages_and_types() -> Result<()> {
    let (article, info) = article(
        r#"<article xml:lang="PT"><front><article-meta>
      <abstract><p>Resumo.</p></abstract>
      <trans-abstract abstract-type="plain-language" xml:lang="EN"><p>English summary.</p></trans-abstract>
      <trans-abstract><p>Inherited language.</p></trans-abstract>
      <trans-abstract xml:lang="" lang="fr"><p>Unspecified language.</p></trans-abstract>
      <abstract abstract-type="graphical"><p>Graphical abstract.</p></abstract>
    </article-meta></front></article>"#,
    )?;
    let parts = article
        .options
        .parts
        .as_ref()
        .ok_or_eyre("missing translations")?;
    assert_eq!(parts.len(), 4);
    for (part, language) in parts.iter().zip([Some("en"), Some("pt"), None, None]) {
        let CreativeWorkVariant::Article(part) = part else {
            bail!("expected an article part")
        };
        assert_eq!(
            part.options
                .extra
                .as_ref()
                .and_then(|extra| extra.get("inLanguage")),
            language
                .map(|language| Primitive::String(language.into()))
                .as_ref()
        );
    }
    let CreativeWorkVariant::Article(part) = &parts[0] else {
        bail!("expected an article part")
    };
    assert_eq!(
        part.options.genre.as_deref(),
        Some(["plain-language".into()].as_slice())
    );
    // An unnamespaced `lang` must not masquerade as `xml:lang`.
    assert_eq!(
        info.losses.iter().collect::<Vec<_>>(),
        [("//article/front/article-meta/trans-abstract/@lang", 1)]
    );
    // The encoder puts every `<abstract>` before any `<trans-abstract>`, as JATS
    // requires, so the parts round-trip once they are in that order
    let (encoded, _) = encode(&Node::Article(article), None)?;
    let (Node::Article(article), _) = decode(&encoded, None)? else {
        bail!("expected an article")
    };
    assert_roundtrip(&article)
}

#[test]
fn reference_ore_credit_role() -> Result<()> {
    let (article, info) = article(
        r#"<article><front><article-meta><contrib-group>
      <contrib contrib-type="author">
        <name><surname>Doe</surname><given-names>Jane</given-names></name>
        <role content-type="http://credit.niso.org/">Writing – Original Draft Preparation</role>
      </contrib>
    </contrib-group></article-meta></front></article>"#,
    )?;
    let authors = article.authors.as_ref().ok_or_eyre("missing author")?;
    assert_eq!(authors.len(), 1);
    let Author::AuthorRole(role) = &authors[0] else {
        bail!("expected a CRediT author role")
    };
    assert_eq!(role.role_name, AuthorRoleName::WritingOriginalDraft);
    let AuthorRoleAuthor::Person(person) = &role.author else {
        bail!("expected a person")
    };
    assert_eq!(
        person.family_names.as_deref(),
        Some(["Doe".into()].as_slice())
    );
    assert!(person.options.job_title.is_none());
    assert!(
        info.losses.is_empty(),
        "{:?}",
        info.losses.iter().collect::<Vec<_>>()
    );
    assert_roundtrip(&article)
}

#[test]
fn reference_ore_orcid_uri() -> Result<()> {
    for identifier in [
        r#"<uri content-type="orcid"> https://orcid.org/0000-0002-1825-0097 </uri>"#,
        r#"<uri content-type="orcid">http://orcid.org/0000-0002-1825-0097</uri>"#,
        r#"<uri content-type="orcid">0000-0002-1825-0097</uri>"#,
        r#"<contrib-id contrib-id-type="orcid">https://orcid.org/0000-0002-1825-0097</contrib-id>
           <uri content-type="orcid">https://orcid.org/0000-0002-1825-0097</uri>"#,
        r#"<uri content-type="orcid"> </uri>
           <contrib-id contrib-id-type="orcid">0000-0002-1825-0097</contrib-id>"#,
        r#"<uri content-type="orcid" xlink:href="https://orcid.org/0000-0002-1825-0097">ORCID</uri>"#,
        r#"<uri content-type="orcid" xlink:href="https://orcid.org/0000-0002-1825-0097/"/>"#,
        r#"<uri content-type="orcid">https://orcid.org/0000-0002-1825-0097/</uri>"#,
        r#"<uri content-type="orcid">orcid.org/0000-0002-1825-0097</uri>"#,
        r#"<object-id xlink:href="https://orcid.org/0000-0002-1825-0097"/>"#,
    ] {
        let (article, info) = article(&format!(
            r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><front><article-meta><contrib-group>
          <contrib contrib-type="author">
            <name><surname>Doe</surname><given-names>Jane</given-names></name>
            {identifier}
          </contrib>
        </contrib-group></article-meta></front></article>"#
        ))?;
        let authors = article.authors.as_ref().ok_or_eyre("missing author")?;
        let Author::Person(person) = &authors[0] else {
            bail!("expected a person")
        };
        assert_eq!(person.orcid.as_deref(), Some("0000-0002-1825-0097"));
        assert!(
            info.losses.is_empty(),
            "{:?}",
            info.losses.iter().collect::<Vec<_>>()
        );
        assert_roundtrip(&article)?;
    }
    Ok(())
}

#[test]
fn reference_orcid_uri_without_id() -> Result<()> {
    // A label is not an iD, so it must not be used to pair people
    let (article, info) = article(
        r#"<article><front><article-meta><contrib-group>
      <contrib contrib-type="author">
        <name><surname>Doe</surname><given-names>Jane</given-names></name>
        <uri content-type="orcid">ORCID</uri>
      </contrib>
    </contrib-group></article-meta></front></article>"#,
    )?;
    let authors = article.authors.as_ref().ok_or_eyre("missing author")?;
    let Author::Person(person) = &authors[0] else {
        bail!("expected a person")
    };
    assert!(person.orcid.is_none());
    assert_eq!(
        info.losses.iter().collect::<Vec<_>>(),
        [("//article/front/article-meta/contrib-group/contrib/uri", 1)]
    );
    Ok(())
}

#[test]
fn reference_funding_statements() -> Result<()> {
    let (article, info) = article(
        r#"<article><front><article-meta>
      <funding-group>
        <award-group><funding-source>Research Council</funding-source><award-id>123</award-id></award-group>
        <funding-statement id="funding">Supported by <italic>Research Council</italic>.</funding-statement>
      </funding-group>
      <funding-group><funding-statement>The funders had no role in the study.</funding-statement></funding-group>
    </article-meta></front><body><p>Body text.</p></body></article>"#,
    )?;
    assert_eq!(article.content.len(), 3);
    assert_eq!(to_text(&article.content[0]).trim(), "Body text.");
    for (block, id, text) in [
        (
            &article.content[1],
            Some("funding"),
            "Supported by Research Council.",
        ),
        (
            &article.content[2],
            None,
            "The funders had no role in the study.",
        ),
    ] {
        let Block::Section(section) = block else {
            bail!("expected a funding section")
        };
        assert_eq!(section.section_type, Some(SectionType::Funding));
        assert_eq!(section.id.as_deref(), id);
        assert_eq!(to_text(&section.content).trim(), text);
    }
    assert_eq!(article.options.funders.as_ref().map(Vec::len), Some(1));
    assert_eq!(article.options.funded_by.as_ref().map(Vec::len), Some(1));
    let Block::Section(section) = &article.content[1] else {
        bail!("expected a funding section")
    };
    let Block::Paragraph(paragraph) = &section.content[0] else {
        bail!("expected a funding paragraph")
    };
    assert!(matches!(
        paragraph.content.get(1),
        Some(Inline::Emphasis(..))
    ));
    assert!(
        info.losses.is_empty(),
        "{:?}",
        info.losses.iter().collect::<Vec<_>>()
    );
    assert_roundtrip(&article)
}

#[test]
fn reference_empty_funding_statements() -> Result<()> {
    let (article, info) = article(
        r#"<article><front><article-meta><funding-group>
      <funding-statement/>
      <funding-statement id="empty"> </funding-statement>
      <funding-statement>Funded.</funding-statement>
    </funding-group></article-meta></front></article>"#,
    )?;
    assert_eq!(article.content.len(), 1);
    assert_eq!(to_text(&article.content[0]).trim(), "Funded.");
    // Only an empty statement with an id loses anything
    assert_eq!(
        info.losses.iter().collect::<Vec<_>>(),
        [(
            "//article/front/article-meta/funding-group/funding-statement",
            1
        )]
    );
    assert_roundtrip(&article)
}

#[test]
fn reference_casrai_credit_role() -> Result<()> {
    for base in ["http://credit.casrai.org/", "https://credit.casrai.org"] {
        let (article, info) = article(&format!(
            r#"<article><front><article-meta><contrib-group>
          <contrib contrib-type="author">
            <name><surname>Doe</surname><given-names>Jane</given-names></name>
            <role content-type="{base}">Writing – Original Draft Preparation</role>
          </contrib>
        </contrib-group></article-meta></front></article>"#
        ))?;
        let authors = article.authors.as_ref().ok_or_eyre("missing author")?;
        let Author::AuthorRole(role) = &authors[0] else {
            bail!("expected a CRediT author role")
        };
        assert_eq!(role.role_name, AuthorRoleName::WritingOriginalDraft);
        assert!(
            info.losses.is_empty(),
            "{:?}",
            info.losses.iter().collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[test]
fn reference_translated_abstract_without_language() -> Result<()> {
    // Without a primary abstract, a translation must not become the primary
    // abstract when re-decoded, nor lose its id
    for attrs in ["", r#" xml:lang="""#] {
        let (article, info) = article(&format!(
            r#"<article><front><article-meta>
          <trans-abstract id="t"{attrs}><p>Summary.</p></trans-abstract>
        </article-meta></front></article>"#
        ))?;
        assert!(article.r#abstract.is_none());
        assert!(info.losses.is_empty());
        let (encoded, info) = encode(&Node::Article(article.clone()), None)?;
        assert!(encoded.contains(r#"<trans-abstract id="t">"#), "{encoded}");
        assert!(info.losses.is_empty());
        assert_roundtrip(&article)?;
    }
    Ok(())
}

#[test]
fn encodes_abstracts_before_translated_abstracts() -> Result<()> {
    let (article, _) = article(
        r#"<article><front><article-meta>
      <abstract><p>Main.</p></abstract>
      <trans-abstract xml:lang="fr"><p>Résumé.</p></trans-abstract>
      <abstract abstract-type="graphical"><p>Graphical.</p></abstract>
    </article-meta></front></article>"#,
    )?;
    let (encoded, _) = encode(&Node::Article(article), None)?;
    let graphical = encoded
        .find(r#"abstract-type="graphical""#)
        .ok_or_eyre("missing graphical abstract")?;
    let translation = encoded
        .find("<trans-abstract")
        .ok_or_eyre("missing translation")?;
    assert!(graphical < translation, "{encoded}");
    Ok(())
}

#[test]
fn sub_article_translated_abstracts() -> Result<()> {
    let (article, info) = article(
        r#"<article><front><article-meta><title-group><article-title>Main</article-title></title-group></article-meta></front>
      <sub-article id="review" article-type="referee-report">
        <front-stub>
          <title-group><article-title>Review</article-title></title-group>
          <abstract><p>Resumo.</p></abstract>
          <trans-abstract id="review-en" xml:lang="en"><p>Summary.</p></trans-abstract>
        </front-stub>
        <body><p>Review text.</p></body>
      </sub-article>
    </article>"#,
    )?;
    assert!(
        info.losses.is_empty(),
        "{:?}",
        info.losses.iter().collect::<Vec<_>>()
    );
    let (encoded, info) = encode(&Node::Article(article.clone()), None)?;
    assert!(encoded.contains("Summary."), "{encoded}");
    assert!(
        info.losses.is_empty(),
        "{:?}",
        info.losses.iter().collect::<Vec<_>>()
    );
    assert_roundtrip(&article)
}

#[test]
fn reports_extra_of_empty_abstract_parts() -> Result<()> {
    let (mut article, _) = article(
        r#"<article><front><article-meta>
      <abstract><p>Main.</p></abstract>
      <trans-abstract xml:lang="fr"><p>Résumé.</p></trans-abstract>
    </article-meta></front></article>"#,
    )?;
    let Some(CreativeWorkVariant::Article(part)) = article
        .options
        .parts
        .as_mut()
        .and_then(|parts| parts.first_mut())
    else {
        bail!("expected an article part")
    };
    part.r#abstract = Some(Vec::new());
    let (_, info) = encode(&Node::Article(article), None)?;
    assert_eq!(
        info.losses.iter().collect::<Vec<_>>(),
        [("Article.extra", 1)]
    );
    Ok(())
}

#[test]
fn retains_distinct_appendices() -> Result<()> {
    let (article, info) = article(
        r#"<article><back><app-group>
          <app id="a"><p>Appendix A</p></app>
          <app id="b"><p>Appendix B</p></app>
        </app-group></back></article>"#,
    )?;
    assert!(info.losses.is_empty());
    assert_eq!(article.content.len(), 2);
    for (block, id, text) in [
        (&article.content[0], "a", "Appendix A"),
        (&article.content[1], "b", "Appendix B"),
    ] {
        let Block::Section(section) = block else {
            bail!("expected an appendix")
        };
        assert_eq!(section.id.as_deref(), Some(id));
        assert_eq!(to_text(&section.content).trim(), text);
    }
    assert_roundtrip(&article)
}

#[test]
fn deduplicates_only_identical_back_matter_sections() -> Result<()> {
    let (article, _) = article(
        r#"<article>
          <body><sec sec-type="acknowledgements"><p>Thanks</p></sec></body>
          <back>
            <ack><p>Thanks</p></ack>
            <ack id="additional"><p>Thanks</p></ack>
            <ack><p>Different thanks</p></ack>
          </back>
        </article>"#,
    )?;
    assert_eq!(article.content.len(), 3);
    assert!(matches!(
        &article.content[1],
        Block::Section(section) if section.id.as_deref() == Some("additional")
    ));
    assert_eq!(to_text(&article.content[2]).trim(), "Different thanks");
    assert_roundtrip(&article)
}

#[test]
fn preserves_images_in_table_cells() -> Result<()> {
    let (article, info) = article(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body>
          <table-wrap><table><tr>
            <td> <inline-graphic xlink:href="inline.png"/> </td>
            <td><p><inline-graphic xlink:href="paragraph.png"/></p></td>
            <td><disp-formula><graphic xlink:href="formula.png"/></disp-formula></td>
            <td><inline-graphic xlink:href="before.png"/> text <inline-graphic xlink:href="after.png"/></td>
            <td> </td>
          </tr></table></table-wrap>
        </body></article>"#,
    )?;
    assert!(info.losses.is_empty());
    let Block::Table(table) = &article.content[0] else {
        bail!("expected a table")
    };
    let cells = &table.rows[0].cells;
    for (cell, url) in [(&cells[0], "inline.png"), (&cells[1], "paragraph.png")] {
        let Block::Paragraph(paragraph) = &cell.content[0] else {
            bail!("expected an image paragraph")
        };
        assert_eq!(paragraph.content.len(), 1);
        assert!(matches!(
            &paragraph.content[0],
            Inline::ImageObject(image) if image.content_url == url
        ));
    }
    let Block::MathBlock(math) = &cells[2].content[0] else {
        bail!("expected an image formula")
    };
    let images = math
        .options
        .images
        .as_ref()
        .ok_or_eyre("missing formula image")?;
    assert_eq!(images[0].content_url, "formula.png");
    let Block::Paragraph(paragraph) = &cells[3].content[0] else {
        bail!("expected a paragraph with images surrounding text")
    };
    assert!(
        matches!(paragraph.content.first(), Some(Inline::ImageObject(image)) if image.content_url == "before.png")
    );
    assert!(
        matches!(paragraph.content.last(), Some(Inline::ImageObject(image)) if image.content_url == "after.png")
    );
    assert!(cells[4].content.is_empty());
    assert_roundtrip(&article)
}

#[test]
fn preserves_inline_runs_around_table_cell_blocks() -> Result<()> {
    let (article, info) = article(
        r#"<article><body><table-wrap><table><tr>
          <th>Before <p>Middle</p> After</th>
          <td><bold>Start</bold><p>One</p> between <p>Two</p><italic>End</italic></td>
        </tr></table></table-wrap></body></article>"#,
    )?;
    assert!(info.losses.is_empty());
    let Block::Table(table) = &article.content[0] else {
        bail!("expected a table")
    };
    for (cell, expected) in [
        (&table.rows[0].cells[0], vec!["Before", "Middle", "After"]),
        (
            &table.rows[0].cells[1],
            vec!["Start", "One", "between", "Two", "End"],
        ),
    ] {
        let actual = cell
            .content
            .iter()
            .map(|block| to_text(block).trim().to_string())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
    assert_roundtrip(&article)
}

#[test]
fn accumulates_reference_lists_and_ignores_empty_lists() -> Result<()> {
    let (article, info) = article(
        r#"<article><back>
          <ref-list><ref id="r1"><element-citation><article-title>First study</article-title></element-citation></ref></ref-list>
          <ref-list><ref id="r2"><element-citation><article-title>Second study</article-title></element-citation></ref></ref-list>
          <ref-list/>
        </back></article>"#,
    )?;
    assert!(info.losses.is_empty());
    let references = article
        .references
        .as_ref()
        .ok_or_eyre("missing references")?;
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("r1"), Some("r2")]
    );
    assert_roundtrip(&article)
}

#[test]
fn retains_boxed_text_caption_after_whitespace_or_comments() -> Result<()> {
    let compact = "<article><body><boxed-text><caption><title>Important</title></caption><p>Content</p></boxed-text></body></article>";
    let (expected, expected_info) = article(compact)?;
    for prefix in ["\n  ", "\n<!-- a comment -->\n  "] {
        let source = compact.replace("<boxed-text>", &format!("<boxed-text>{prefix}"));
        let (actual, info) = article(&source)?;
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::to_value(&info.losses)?,
            serde_json::to_value(&expected_info.losses)?
        );
        let Block::Admonition(admonition) = &actual.content[0] else {
            bail!("expected boxed text")
        };
        assert_eq!(
            to_text(admonition.title.as_ref().ok_or_eyre("missing caption")?),
            "Important"
        );
        assert_roundtrip(&actual)?;
    }
    Ok(())
}

fn assert_mathml_preserved(mathml: &str) -> Result<()> {
    let (article, info) = article(&format!(
        "<article><body><disp-formula>{mathml}</disp-formula></body></article>"
    ))?;
    assert!(info.losses.is_empty());
    let Block::MathBlock(math) = &article.content[0] else {
        bail!("expected a formula")
    };
    assert_xml_elements_equal(mathml, &math.code)?;

    // Pretty encoding can indent the MathML source. Compare expanded names
    // and attribute values rather than requiring identical source whitespace.
    let node = Node::Article(article);
    for compact in [true, false] {
        let (encoded, _) = encode(
            &node,
            Some(EncodeOptions {
                compact: Some(compact),
                ..Default::default()
            }),
        )?;
        let (decoded, _) = decode(&encoded, None)?;
        let Node::Article(article) = decoded else {
            bail!("expected an article")
        };
        let Block::MathBlock(math) = &article.content[0] else {
            bail!("expected a formula")
        };
        assert_xml_elements_equal(mathml, &math.code)?;
    }
    Ok(())
}

fn assert_xml_elements_equal(expected: &str, actual: &str) -> Result<()> {
    let expected = roxmltree::Document::parse(expected)?;
    let actual = roxmltree::Document::parse(actual)?;
    assert_xml_nodes_equal(&expected.root_element(), &actual.root_element());
    Ok(())
}

/// Assert that two elements have the same names, attributes, text and
/// structure, regardless of the namespace prefixes used for them
///
/// Whitespace between MathML elements is not significant, and pretty encoding
/// indents it, so text that is only whitespace is ignored.
fn assert_xml_nodes_equal(expected: &roxmltree::Node, actual: &roxmltree::Node) {
    assert_eq!(actual.tag_name(), expected.tag_name());
    assert_eq!(actual.attributes().len(), expected.attributes().len());
    for attribute in expected.attributes() {
        let value = actual
            .attributes()
            .find(|actual| {
                actual.name() == attribute.name() && actual.namespace() == attribute.namespace()
            })
            .map(|attribute| attribute.value());
        assert_eq!(
            value,
            Some(attribute.value()),
            "attribute {}",
            attribute.name()
        );
    }

    let is_content = |child: &roxmltree::Node| {
        child.is_element()
            || child
                .text()
                .is_some_and(|text| child.is_text() && !text.trim().is_empty())
    };
    let expected_children = expected.children().filter(is_content).collect::<Vec<_>>();
    let actual_children = actual.children().filter(is_content).collect::<Vec<_>>();
    assert_eq!(
        actual_children.len(),
        expected_children.len(),
        "children of {:?}",
        expected.tag_name()
    );
    for (actual, expected) in actual_children.iter().zip(&expected_children) {
        if expected.is_text() {
            assert_eq!(actual.text().map(str::trim), expected.text().map(str::trim));
        } else {
            assert_xml_nodes_equal(expected, actual);
        }
    }
}

#[test]
fn xml_comparison_detects_changed_text_and_nesting() {
    let compare = |expected: &str, actual: &str| {
        std::panic::catch_unwind(|| assert_xml_elements_equal(expected, actual)).is_err()
    };
    assert!(compare(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi></math>"#,
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mi>y</mi></math>"#
    ));
    assert!(compare(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mrow><mi>x</mi></mrow><mi>y</mi></math>"#,
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mrow><mi>x</mi><mi>y</mi></mrow></math>"#
    ));
}

#[test]
fn escapes_mathml_attribute_values() -> Result<()> {
    assert_mathml_preserved(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML" alttext="A &amp; B &lt; C &quot;quoted&quot; &apos;single&apos; &#9;&#10;&#13;"><mi>x</mi></math>"#,
    )
}

#[test]
fn preserves_scoped_default_namespaces_in_mathml() -> Result<()> {
    assert_mathml_preserved(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mi>x</mi><annotation-xml encoding="application/xhtml+xml"><div xmlns="http://www.w3.org/1999/xhtml"><span>x</span><plain xmlns="">y</plain></div></annotation-xml></semantics></math>"#,
    )
}

#[test]
fn preserves_rebound_prefixes_and_namespaced_attributes_in_mathml() -> Result<()> {
    assert_mathml_preserved(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML" xmlns:m="http://www.w3.org/1998/Math/MathML" xmlns:q="urn:outer" m:flag="yes" xml:lang="en"><semantics><mi q:flag="outer">x</mi><annotation-xml><q:outer><q:inner xmlns:q="urn:inner" q:flag="inner"/><q:outer/></q:outer></annotation-xml></semantics></math>"#,
    )
}

/// Wrap body content in an article
fn body(content: &str) -> String {
    format!(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><body>{content}</body></article>"#
    )
}

/// Wrap article metadata in an article
fn meta(content: &str) -> String {
    format!(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><front><article-meta>{content}</article-meta></front></article>"#
    )
}

/// Wrap a reference list in an article
fn refs(content: &str) -> String {
    format!(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><back><ref-list>{content}</ref-list></back></article>"#
    )
}

fn has_loss(info: &DecodeInfo, label: &str) -> bool {
    info.losses.iter().any(|(loss, ..)| loss == label)
}

/// The labels of losses, for assertion messages
fn labels(losses: &stencila_codec::Losses) -> Vec<&str> {
    losses.iter().map(|(label, ..)| label).collect()
}

#[test]
fn inline_code_in_table_cell_stays_inline() -> Result<()> {
    let (article, _) = article(&body(
        "<table-wrap><table><tbody><tr><td>Use <code>--foo</code> flag</td></tr></tbody></table></table-wrap>",
    ))?;
    let Some(Block::Table(table)) = article.content.first() else {
        bail!("expected a table")
    };
    let cell = &table.rows[0].cells[0];
    assert_eq!(cell.content.len(), 1);
    let Block::Paragraph(paragraph) = &cell.content[0] else {
        bail!("expected a paragraph")
    };
    assert!(matches!(paragraph.content[1], Inline::CodeInline(..)));
    Ok(())
}

#[test]
fn only_executable_yes_code_is_a_chunk() -> Result<()> {
    for (attribute, is_chunk) in [
        (r#" executable="yes""#, true),
        (r#" executable="no""#, false),
        ("", false),
    ] {
        let (article, info) = article(&body(&format!(
            r#"<code language="python"{attribute}>print(1)</code>"#
        )))?;
        assert_eq!(
            matches!(article.content.first(), Some(Block::CodeChunk(..))),
            is_chunk,
            "{attribute}"
        );
        assert!(
            info.losses.is_empty(),
            "{attribute}: {:?}",
            labels(&info.losses)
        );
    }

    let (article, info) = self::article(&body(r#"<code executable="maybe">x</code>"#))?;
    assert!(matches!(
        article.content.first(),
        Some(Block::CodeBlock(..))
    ));
    assert!(has_loss(&info, "//article/body/code/@executable"));
    Ok(())
}

#[test]
fn code_is_not_truncated_by_comments_or_labels() -> Result<()> {
    let (article, _) = article(&body(
        r#"<code language="python">x = 1<!-- note --> + 2</code><code executable="yes"><label>Code 1</label>y = 2</code>"#,
    ))?;
    let [Block::CodeBlock(block), Block::CodeChunk(chunk)] = article.content.as_slice() else {
        bail!("expected a code block and chunk")
    };
    assert_eq!(block.code.as_str(), "x = 1 + 2");
    assert_eq!(chunk.code.as_str(), "y = 2");
    assert_eq!(chunk.label.as_deref(), Some("Code 1"));
    Ok(())
}

#[test]
fn textual_and_composite_formulas_keep_their_math() -> Result<()> {
    let (article, info) = article(&body(
        "<p><inline-formula><italic>x</italic><sup>2</sup> + 1</inline-formula></p>",
    ))?;
    let Some(Block::Paragraph(paragraph)) = article.content.first() else {
        bail!("expected a paragraph")
    };
    let Inline::MathInline(math) = &paragraph.content[0] else {
        bail!("expected math")
    };
    assert_eq!(math.code.as_str(), "x^{2} + 1");
    assert_eq!(math.math_language.as_deref(), Some("tex"));
    assert!(info.losses.is_empty(), "{:?}", labels(&info.losses));

    let (composite, _) = self::article(&body(
        "<disp-formula><mml:math><mml:mi>x</mml:mi></mml:math> + <mml:math><mml:mi>y</mml:mi></mml:math></disp-formula>",
    ))?;
    let Some(Block::MathBlock(math)) = composite.content.first() else {
        bail!("expected math")
    };
    assert_eq!(math.math_language.as_deref(), Some("mathml"));
    assert!(math.code.contains(">x<") && math.code.contains(">+<") && math.code.contains(">y<"));
    roxmltree::Document::parse(&math.code)?;
    Ok(())
}

#[test]
fn references_are_kept_wherever_they_are() -> Result<()> {
    let citation = |title: &str| {
        format!(
            r#"<element-citation publication-type="journal"><article-title>{title}</article-title></element-citation>"#
        )
    };
    let (article, info) = article(&format!(
        r#"<article><body><sec><title>Methods</title><ref-list><ref id="m1">{}</ref></ref-list></sec></body><back><ref-list><ref>{}</ref><ref-list><ref id="n1">{}</ref></ref-list><ref id="o1"><nlm-citation citation-type="journal"><article-title>Old</article-title></nlm-citation></ref></ref-list></back></article>"#,
        citation("In section"),
        citation("No id"),
        citation("Nested"),
    ))?;
    let references = article.references.as_ref().ok_or_eyre("no references")?;
    let titles = references
        .iter()
        .map(|reference| to_text(&reference.title))
        .collect::<Vec<_>>();
    assert_eq!(titles, ["In section", "No id", "Nested", "Old"]);
    assert_eq!(references[1].id, None);
    assert!(info.losses.is_empty(), "{:?}", labels(&info.losses));
    Ok(())
}

#[test]
fn string_names_are_decoded() -> Result<()> {
    let (article, _) = article(
        r#"<article><front><article-meta><contrib-group><contrib contrib-type="author"><string-name><given-names>Jane</given-names> <surname>Doe</surname></string-name></contrib><contrib contrib-type="author"><string-name>John Q. Public</string-name></contrib></contrib-group></article-meta></front><back><ref-list><ref id="r1"><mixed-citation><string-name>Smith J</string-name>. A title. 2020.</mixed-citation></ref></ref-list></back></article>"#,
    )?;
    let authors = article.authors.as_ref().ok_or_eyre("no authors")?;
    let [Author::Person(jane), Author::Person(john)] = authors.as_slice() else {
        bail!("expected two people")
    };
    assert_eq!(jane.family_names.as_deref(), Some(&["Doe".to_string()][..]));
    assert_eq!(john.options.name.as_deref(), Some("John Q. Public"));

    let references = article.references.as_ref().ok_or_eyre("no references")?;
    let Some(Author::Person(smith)) = references[0].authors.as_ref().and_then(|a| a.first()) else {
        bail!("expected a person")
    };
    assert_eq!(smith.options.name.as_deref(), Some("Smith J"));
    Ok(())
}

#[test]
fn keywords_keep_all_their_text() -> Result<()> {
    let (article, _) = article(&meta(
        "<kwd-group><kwd><italic>S. cerevisiae</italic> budding yeast</kwd><kwd>\n  pretty keyword\n</kwd></kwd-group><kwd-group/>",
    ))?;
    assert_eq!(
        article.options.keywords,
        Some(vec![
            "S. cerevisiae budding yeast".to_string(),
            "pretty keyword".to_string()
        ])
    );
    Ok(())
}

#[test]
fn supplement_types_from_files_and_roundtrip() -> Result<()> {
    let (article, _) = article(&body(
        r#"<supplementary-material><label>Supplementary Table 1</label><media xlink:href="supplements/file02.xlsx"/></supplementary-material><supplementary-material><media xlink:href="data.csv?download=1"/></supplementary-material><supplementary-material><label>Album</label><media xlink:href="x.bin"/></supplementary-material>"#,
    ))?;
    let supplements = article
        .content
        .iter()
        .filter_map(|block| match block {
            Block::Supplement(supplement) => Some(supplement),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(supplements[0].work_type, Some(CreativeWorkType::Datatable));
    assert_eq!(supplements[0].label.as_deref(), Some("1"));
    assert_eq!(supplements[1].work_type, Some(CreativeWorkType::Datatable));
    assert_eq!(supplements[2].label.as_deref(), Some("Album"));
    assert_roundtrip(&article)
}

#[test]
fn pretty_encoding_does_not_add_whitespace_to_metadata() -> Result<()> {
    let (article, _) = article(&refs(
        r#"<ref id="r1"><element-citation publication-type="journal"><article-title>A study</article-title><source><italic>Nature</italic></source><year>2020</year></element-citation></ref>"#,
    ))?;
    assert_roundtrip(&article)
}

#[test]
fn title_group_extras_are_reported() -> Result<()> {
    let (article, info) = article(&meta(
        r#"<title-group><article-title>Main</article-title><subtitle>Sub</subtitle><trans-title-group xml:lang="fr"><trans-title>Principal</trans-title></trans-title-group></title-group>"#,
    ))?;
    assert_eq!(to_text(&article.title), "Main");
    assert!(has_loss(
        &info,
        "//article/front/article-meta/title-group/subtitle"
    ));
    assert!(has_loss(
        &info,
        "//article/front/article-meta/title-group/trans-title-group"
    ));
    Ok(())
}

#[test]
fn root_article_type_and_language_roundtrip() -> Result<()> {
    let (article, info) = article(
        r#"<article article-type="retraction" xml:lang="fr" dtd-version="1.3"><body><p>Text</p></body></article>"#,
    )?;
    assert_eq!(article.options.genre, Some(vec!["retraction".to_string()]));
    assert!(info.losses.is_empty(), "{:?}", labels(&info.losses));
    let (jats, encoded) = encode(&Node::Article(article.clone()), None)?;
    assert!(jats.contains(r#"article-type="retraction""#), "{jats}");
    assert!(jats.contains(r#"xml:lang="fr""#), "{jats}");
    assert!(encoded.losses.is_empty(), "{:?}", labels(&encoded.losses));
    assert_roundtrip(&article)
}

#[test]
fn translated_citation_titles_do_not_replace_titles() -> Result<()> {
    let (article, info) = article(&refs(
        r#"<ref id="r1"><element-citation><article-title>Original</article-title><trans-title>Translated</trans-title></element-citation></ref><ref id="r2"><element-citation><trans-title>Only translated</trans-title></element-citation></ref>"#,
    ))?;
    let references = article.references.as_ref().ok_or_eyre("no references")?;
    assert_eq!(to_text(&references[0].title), "Original");
    assert_eq!(to_text(&references[1].title), "Only translated");
    assert!(has_loss(
        &info,
        "//article/back/ref-list/ref/element-citation/trans-title"
    ));
    Ok(())
}

#[test]
fn person_groups_are_not_all_editors() -> Result<()> {
    let name = "<name><surname>Doe</surname><given-names>J</given-names></name>";
    let (article, info) = article(&refs(&format!(
        r#"<ref id="r1"><element-citation><person-group person-group-type="inventor">{name}</person-group><person-group person-group-type="guest-editor">{name}</person-group><article-title>T</article-title></element-citation></ref>"#
    )))?;
    let reference = &article.references.as_ref().ok_or_eyre("no references")?[0];
    assert!(reference.authors.is_none());
    assert_eq!(reference.options.editors.as_ref().map(Vec::len), Some(1));
    assert!(has_loss(
        &info,
        "//article/back/ref-list/ref/element-citation/person-group"
    ));
    assert!(has_loss(
        &info,
        "//article/back/ref-list/ref/element-citation/person-group/@person-group-type"
    ));
    Ok(())
}

#[test]
fn publication_dates_keep_type_and_format() -> Result<()> {
    let (article, _) = article(&meta(
        r#"<pub-date date-type="pub" publication-format="print" iso-8601-date="2025-06-01"/><pub-date date-type="pub" publication-format="electronic" iso-8601-date="2025-05-01"/>"#,
    ))?;
    assert_eq!(
        article
            .date_published
            .as_ref()
            .map(|date| date.value.as_str()),
        Some("2025-05-01")
    );
    let (jats, _) = encode(&Node::Article(article.clone()), None)?;
    assert!(
        jats.contains(r#"<pub-date date-type="pub" publication-format="electronic""#),
        "{jats}"
    );
    assert_roundtrip(&article)
}

#[test]
fn repeated_volumes_keep_the_journal() -> Result<()> {
    let (article, info) = article(
        r#"<article><front><journal-meta><journal-title>J Foo</journal-title></journal-meta><article-meta><issue>3</issue><volume>12</volume><volume content-type="online">12e</volume><fpage>056</fpage></article-meta></front></article>"#,
    )?;
    let Some(CreativeWorkVariant::PublicationIssue(issue)) = &article.options.is_part_of else {
        bail!("expected an issue")
    };
    let Some(CreativeWorkVariant::PublicationVolume(volume)) = issue.is_part_of.as_deref() else {
        bail!("expected a volume")
    };
    assert_eq!(volume.volume_number, Some(IntegerOrString::Integer(12)));
    assert!(matches!(
        volume.is_part_of.as_deref(),
        Some(CreativeWorkVariant::Periodical(..))
    ));
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::String("056".into()))
    );
    assert!(has_loss(&info, "//article/front/article-meta/volume"));
    Ok(())
}

#[test]
fn collaboration_members_are_not_part_of_its_name() -> Result<()> {
    let (article, _) = article(&meta(
        "<contrib-group><contrib><collab>The <italic>Big</italic> Consortium<contrib-group><contrib><name><surname>Member</surname><given-names>M</given-names></name></contrib></contrib-group></collab></contrib></contrib-group>",
    ))?;
    let Some(Author::Organization(org)) = article.authors.as_ref().and_then(|a| a.first()) else {
        bail!("expected an organization")
    };
    assert_eq!(org.name.as_deref(), Some("The Big Consortium"));
    assert_eq!(org.options.members.as_ref().map(Vec::len), Some(1));
    Ok(())
}

#[test]
fn single_group_affiliation_applies_to_all() -> Result<()> {
    let (article, info) = article(&meta(
        "<contrib-group><contrib><name><surname>A</surname></name></contrib><contrib><name><surname>B</surname></name></contrib><aff>Example University</aff></contrib-group>",
    ))?;
    let authors = article.authors.as_ref().ok_or_eyre("no authors")?;
    for author in authors {
        let Author::Person(person) = author else {
            bail!("expected a person")
        };
        assert_eq!(person.affiliations.as_ref().map(Vec::len), Some(1));
    }
    assert!(info.losses.is_empty(), "{:?}", labels(&info.losses));
    Ok(())
}

#[test]
fn alternatives_are_not_dropped_or_concatenated() -> Result<()> {
    let (article, _) = article(&body(
        r#"<fig id="f1"><alternatives><graphic xlink:href="figure.tif"/><graphic xlink:href="figure.jpg"/></alternatives></fig><table-wrap id="t1"><alternatives><graphic xlink:href="table.png"/><table><tbody><tr><td>1</td></tr></tbody></table><table><tbody><tr><td>1</td></tr></tbody></table></alternatives></table-wrap>"#,
    ))?;
    let [Block::Figure(figure), Block::Table(table)] = article.content.as_slice() else {
        bail!("expected a figure and table")
    };
    assert_eq!(figure.content.len(), 1);
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.options.images.as_ref().map(Vec::len), Some(1));
    assert_roundtrip(&article)
}

#[test]
fn paragraph_ids_are_kept_and_linkable() -> Result<()> {
    let (article, info) = article(&body(
        r##"<p id="p1">First</p><p>See <xref rid="p1">above</xref> and <xref rid="f1">Figure 1</xref>.</p><fig id="f1"><graphic xlink:href="f.png"/></fig>"##,
    ))?;
    assert!(info.losses.is_empty(), "{:?}", labels(&info.losses));
    let Some(Block::Paragraph(first)) = article.content.first() else {
        bail!("expected a paragraph")
    };
    assert_eq!(first.id.as_deref(), Some("p1"));
    let Some(Block::Paragraph(second)) = article.content.get(1) else {
        bail!("expected a paragraph")
    };
    let links = second
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Link(link) => Some(link.target.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(links, ["#p1", "#f1"]);
    let (_, encoded) = encode(&Node::Article(article.clone()), None)?;
    assert!(
        !encoded
            .losses
            .iter()
            .any(|(loss, ..)| loss == "Link.target"),
        "{:?}",
        labels(&encoded.losses)
    );
    Ok(())
}

#[test]
fn list_types_and_children_are_reported() -> Result<()> {
    let (_, info) = article(&body(
        r#"<list list-type="simple"><title>Items</title><list-item><p>A</p></list-item></list>"#,
    ))?;
    assert!(has_loss(&info, "//article/body/list/@list-type"));
    assert!(has_loss(&info, "//article/body/list/title"));
    Ok(())
}

#[test]
fn jats_section_types_are_mapped() -> Result<()> {
    let (article, info) = article(&body(
        r#"<sec sec-type="materials|methods"><title>M</title><p>x</p></sec><sec sec-type="additional-information"><title>Additional</title><p>y</p></sec>"#,
    ))?;
    let Some(Block::Section(section)) = article.content.first() else {
        bail!("expected a section")
    };
    assert_eq!(section.section_type, Some(SectionType::Materials));
    assert!(has_loss(&info, "//article/body/sec/@sec-type"));
    Ok(())
}

#[test]
fn uris_are_links_and_publishers_are_complete() -> Result<()> {
    let (article, _) = article(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><front><journal-meta><publisher><publisher-name>A <italic>B</italic> Press</publisher-name></publisher></journal-meta></front><body><p>See <uri>https://example.org</uri>.</p></body></article>"#,
    )?;
    let Some(PersonOrOrganization::Organization(publisher)) = &article.options.publisher else {
        bail!("expected a publisher")
    };
    assert_eq!(publisher.name.as_deref(), Some("A B Press"));
    let Some(Block::Paragraph(paragraph)) = article.content.first() else {
        bail!("expected a paragraph")
    };
    let Inline::Link(link) = &paragraph.content[1] else {
        bail!("expected a link")
    };
    assert_eq!(link.target, "https://example.org");
    Ok(())
}

#[test]
fn funding_source_losses_have_its_path() -> Result<()> {
    let (_, info) = article(&meta(
        r#"<funding-group><award-group><funding-source country="NZ">Fund</funding-source><award-id>1</award-id></award-group></funding-group>"#,
    ))?;
    assert!(has_loss(
        &info,
        "//article/front/article-meta/funding-group/award-group/funding-source/@country"
    ));
    Ok(())
}

#[test]
fn additional_articles_in_a_set_are_reported() -> Result<()> {
    let (_, info) = article(
        "<pmc-articleset><article><body><p>A</p></body></article><article><body><p>B</p></body></article></pmc-articleset>",
    )?;
    assert!(has_loss(&info, "//pmc-articleset/article"));
    Ok(())
}
