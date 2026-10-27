use stencila_codec::{
    DecodeInfo, EncodeOptions,
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Article, Author, AuthorRoleAuthor, AuthorRoleName, Block, CreativeWorkVariant, Inline,
        Node, Primitive,
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
    let expected = expected
        .descendants()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    let actual = actual
        .descendants()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(&expected) {
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
    }
    Ok(())
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
