use pretty_assertions::assert_eq;
use stencila_codec::eyre::bail;
use stencila_codec::stencila_schema::{
    Article, Author, Block, CreativeWorkType, CreativeWorkVariant, CreativeWorkVariantOrString,
    Date, DateTime, Inline, IntegerOrString, Node, NoteType, Organization, Person,
    PropertyValueOrString, Reference, SectionType,
    shortcuts::{art, aud, h1, img, p, sec, sti, t, vid},
};
use stencila_codec_text_trait::to_text;

use super::*;

/// Roundtrip test for media objects
#[tokio::test]
async fn media_objects() -> Result<()> {
    let codec = JatsCodec {};

    let doc1 = art([p([
        aud("http://example.org/audio.mp3"),
        img("http://example.org/image.png"),
        vid("http://example.org/video.mp4"),
    ])]);

    let (jats, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.4" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><front><article-meta></article-meta></front><body><p><inline-media xlink:href="http://example.org/audio.mp3" mimetype="audio"></inline-media><inline-graphic xlink:href="http://example.org/image.png"></inline-graphic><inline-media xlink:href="http://example.org/video.mp4" mimetype="video"></inline-media></p></body></article>"#
    );

    let (doc2, ..) = codec.from_str(&jats, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}

/// Roundtrip test for `Span`
///
/// This is a regression test for a bug found in roundtrip proptests.
#[tokio::test]
async fn spans() -> Result<()> {
    let codec = JatsCodec {};

    // Code contains whitespace characters that need to be escaped
    let doc1 = art([p([sti("\t\n\r", [])])]);

    let (jats, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.4" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><front><article-meta></article-meta></front><body><p><styled-content style="&#9;&#10;&#13;"></styled-content></p></body></article>"#
    );

    let (doc2, ..) = codec.from_str(&jats, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}

/// Correspondence notes referenced from a contributor should be retained on the person.
#[tokio::test]
async fn contributor_correspondence_email() -> Result<()> {
    let jats = r#"
        <article>
          <front>
            <article-meta>
              <contrib-group>
                <contrib contrib-type="author" corresp="yes">
                  <name><surname>Doe</surname><given-names>Jane</given-names></name>
                  <xref ref-type="corresp" rid="cor1">*</xref>
                </contrib>
              </contrib-group>
              <author-notes>
                <corresp id="cor1">Correspondence: <email>jane@example.org</email></corresp>
              </author-notes>
            </article-meta>
          </front>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let Node::Article(article) = node else {
        return Err(stencila_codec::eyre::eyre!("expected an article"));
    };
    let Some(Author::Person(person)) = article
        .authors
        .and_then(|authors| authors.into_iter().next())
    else {
        return Err(stencila_codec::eyre::eyre!("expected a person author"));
    };

    assert_eq!(person.options.emails, Some(vec!["jane@example.org".into()]));

    Ok(())
}

/// Front and back matter should be emitted in the locations expected by JATS consumers.
#[tokio::test]
async fn article_front_and_back_matter() -> Result<()> {
    let mut affiliation = Organization::new();
    affiliation.name = Some("Example University".into());
    affiliation.ror = Some("03yrm5c26".into());

    let mut person = Person::new();
    person.given_names = Some(vec!["Jane".into()]);
    person.family_names = Some(vec!["Doe".into()]);
    person.orcid = Some("0000-0002-1825-0097".into());
    person.affiliations = Some(vec![affiliation]);
    person.options.emails = Some(vec!["jane@example.org".into()]);
    let mut second_person = person.clone();
    second_person.given_names = Some(vec!["John".into()]);
    second_person.family_names = Some(vec!["Roe".into()]);
    second_person.orcid = None;
    second_person.options.emails = None;

    let mut reference = Reference::new();
    reference.id = Some("doe-2024".into());
    reference.work_type = Some(CreativeWorkType::Article);
    reference.authors = Some(vec![Author::Person(person.clone())]);
    reference.date = Some(Date::new("2024-02-03".into()));
    reference.title = Some(vec![t("Referenced work")]);
    reference.doi = Some("https://doi.org/10.1234/example".into());
    reference.options.text = Some("Doe (2024). Referenced work.".into());

    let mut acknowledgement = match sec([h1([t("Acknowledgements")]), p([t("Thanks")])]) {
        stencila_codec::stencila_schema::Block::Section(section) => section,
        _ => return Err(stencila_codec::eyre::eyre!("expected section")),
    };
    acknowledgement.section_type = Some(SectionType::Acknowledgements);

    let mut appendix = match sec([h1([t("Appendix A")]), p([t("Details")])]) {
        stencila_codec::stencila_schema::Block::Section(section) => section,
        _ => return Err(stencila_codec::eyre::eyre!("expected section")),
    };
    appendix.section_type = Some(SectionType::Appendix);

    let mut article = Article::new(vec![
        sec([h1([t("Introduction")]), p([t("Body")])]),
        Block::Section(acknowledgement),
        Block::Section(appendix),
    ]);
    article.id = Some("article-1".into());
    article.doi = Some("doi:10.5678/article".into());
    article.title = Some(vec![t("Article title")]);
    article.r#abstract = Some(vec![p([t("Summary")])]);
    article.authors = Some(vec![Author::Person(person), Author::Person(second_person)]);
    article.date_published = Some(DateTime::new("2025-04-05T12:00:00Z".into()));
    article.references = Some(vec![reference]);
    article.options.date_received = Some(DateTime::new("2025-01-02".into()));
    article.options.date_accepted = Some(DateTime::new("2025-03-04".into()));
    article.options.keywords = Some(vec!["testing".into(), "JATS".into()]);

    let (jats, info) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    let document = roxmltree::Document::parse(&jats)?;
    let root = document.root_element();
    assert_eq!(root.attribute("dtd-version"), Some("1.4"));
    assert_eq!(root.attribute("id"), Some("article-1"));

    let article_meta = root
        .descendants()
        .find(|node| node.has_tag_name("article-meta"))
        .ok_or_else(|| stencila_codec::eyre::eyre!("missing article-meta"))?;
    let child_names = article_meta
        .children()
        .filter(|node| node.is_element())
        .map(|node| node.tag_name().name())
        .collect::<Vec<_>>();
    assert_eq!(
        child_names,
        [
            "article-id",
            "title-group",
            "contrib-group",
            "pub-date",
            "history",
            "abstract",
            "kwd-group"
        ]
    );
    assert_eq!(
        article_meta
            .descendants()
            .find(|node| node.has_tag_name("article-title"))
            .and_then(|node| node.text()),
        Some("Article title")
    );
    assert_eq!(
        article_meta
            .descendants()
            .find(|node| node.has_tag_name("surname"))
            .and_then(|node| node.text()),
        Some("Doe")
    );
    assert_eq!(
        article_meta
            .descendants()
            .find(|node| node.has_tag_name("aff"))
            .and_then(|node| node.attribute("id")),
        Some("aff1")
    );
    assert_eq!(
        article_meta
            .descendants()
            .filter(|node| node.has_tag_name("aff"))
            .count(),
        1
    );
    assert_eq!(
        article_meta
            .descendants()
            .filter(|node| {
                node.has_tag_name("xref")
                    && node.attribute("ref-type") == Some("aff")
                    && node.attribute("rid") == Some("aff1")
            })
            .count(),
        2
    );
    assert!(root.descendants().any(|node| node.has_tag_name("ack")));
    assert!(root.descendants().any(|node| node.has_tag_name("app")));

    let citation = root
        .descendants()
        .find(|node| node.has_tag_name("mixed-citation"))
        .ok_or_else(|| stencila_codec::eyre::eyre!("missing mixed-citation"))?;
    assert_eq!(citation.attribute("publication-type"), Some("journal"));
    assert_eq!(
        citation
            .descendants()
            .find(|node| node.has_tag_name("pub-id"))
            .and_then(|node| node.text()),
        Some("10.1234/example")
    );
    assert!(
        citation
            .text()
            .is_some_and(|text| text.contains("Doe (2024)"))
    );
    // A <person-group> in a citation carries only the name, so the reference
    // author's other details are reported as lost rather than silently dropped
    assert_eq!(
        info.losses.iter().collect::<Vec<_>>(),
        [
            ("Person.affiliations", 1),
            ("Person.emails", 1),
            ("Person.orcid", 1)
        ]
    );

    Ok(())
}

#[tokio::test]
async fn raw_reference_and_standalone_doctype() -> Result<()> {
    let mut reference = Reference::new();
    reference.options.text = Some("An unstructured reference".into());

    let mut article = Article::new(Vec::new());
    article.references = Some(vec![reference]);

    let (jats, ..) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                standalone: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    assert!(jats.starts_with(concat!(
        "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\" ?>\n",
        "<!DOCTYPE article SYSTEM \"https://jats.nlm.nih.gov/archiving/1.4/",
        "JATS-archivearticle1-4-mathml3.dtd\">\n"
    )));
    assert!(jats.contains("<front><article-meta></article-meta></front>"));
    assert!(jats.contains(
        "<ref id=\"ref1\"><mixed-citation>An unstructured reference</mixed-citation></ref>"
    ));
    Ok(())
}

#[tokio::test]
async fn front_and_back_matter_roundtrip() -> Result<()> {
    let source = r#"
        <article dtd-version="1.4">
          <front>
            <article-meta>
              <article-id pub-id-type="doi">10.5678/article</article-id>
              <title-group><article-title>Roundtrip title</article-title></title-group>
              <contrib-group>
                <contrib contrib-type="author">
                  <name><surname>Doe</surname><given-names>Jane</given-names></name>
                  <contrib-id contrib-id-type="orcid">https://orcid.org/0000-0002-1825-0097</contrib-id>
                  <xref ref-type="aff" rid="aff1"/>
                </contrib>
                <aff id="aff1"><institution>Example University</institution></aff>
              </contrib-group>
              <pub-date iso-8601-date="2025-04-05"><year>2025</year><month>04</month><day>05</day></pub-date>
              <history><date date-type="accepted" iso-8601-date="2025-03-04"><year>2025</year><month>03</month><day>04</day></date></history>
              <abstract><p>Roundtrip summary</p></abstract>
              <kwd-group><kwd>testing</kwd></kwd-group>
            </article-meta>
          </front>
          <body><sec><title>Introduction</title><p>Body</p></sec></body>
          <back>
            <ack><p>Thanks</p></ack>
            <ref-list>
              <ref id="ref1">
                <element-citation publication-type="journal">
                  <person-group person-group-type="author"><name><surname>Smith</surname><given-names>Alex</given-names></name></person-group>
                  <article-title>Referenced work</article-title>
                  <year>2024</year>
                  <pub-id pub-id-type="doi">10.1234/reference</pub-id>
                </element-citation>
              </ref>
            </ref-list>
          </back>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (encoded, info) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(info.losses.is_empty());

    let document = roxmltree::Document::parse(&encoded)?;
    let root = document.root_element();
    for (element, expected) in [
        ("article-id", "10.5678/article"),
        ("article-title", "Roundtrip title"),
        ("surname", "Doe"),
        ("institution", "Example University"),
        ("abstract", "Roundtrip summary"),
        ("kwd", "testing"),
        ("ack", "Thanks"),
    ] {
        let actual = root
            .descendants()
            .find(|node| node.has_tag_name(element))
            .map(|node| {
                node.descendants()
                    .filter_map(|node| node.is_text().then(|| node.text()).flatten())
                    .collect::<String>()
            });
        assert_eq!(
            actual.as_deref(),
            Some(expected),
            "missing or changed {element}"
        );
    }
    let reference = root
        .descendants()
        .find(|node| node.has_tag_name("ref"))
        .ok_or_else(|| stencila_codec::eyre::eyre!("missing ref"))?;
    assert!(reference.descendants().any(|node| {
        node.has_tag_name("article-title") && node.text() == Some("Referenced work")
    }));
    assert!(
        reference.descendants().any(|node| {
            node.has_tag_name("pub-id") && node.text() == Some("10.1234/reference")
        })
    );

    Ok(())
}

/// Losses should be labelled by what they are, not only how many there are
#[test]
fn loss_labels_are_classified_by_semantic_impact() {
    use crate::LossCategory;

    for (label, expected) in [
        ("//article/body/sec/p/list", LossCategory::Content),
        (
            "//article/back/ref-list/ref/@id",
            LossCategory::LinkOrIdentifier,
        ),
        (
            "//article/body/sec/table-wrap/table/tr/td/@style",
            LossCategory::Presentation,
        ),
        (
            "//article/front/article-meta/funding-group",
            LossCategory::Metadata,
        ),
        ("Article.licenses", LossCategory::Metadata),
        ("Reference.identifiers", LossCategory::LinkOrIdentifier),
    ] {
        assert_eq!(crate::classify(label), expected, "misclassified {label}");
    }
}

/// Table cells containing blocks should not report the blocks as lost
///
/// Decoding a cell as inlines first and only then falling back to blocks
/// reported every `<p>` in every cell as lost, even though it was decoded.
#[tokio::test]
async fn table_cell_blocks_are_not_reported_as_lost() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <table-wrap id="tbl1">
              <table>
                <thead><tr><th><p>Header</p></th></tr></thead>
                <tbody>
                  <tr><td><p>First paragraph</p><p>Second paragraph</p></td></tr>
                  <tr><td>Plain <italic>inline</italic> text</td></tr>
                </tbody>
              </table>
            </table-wrap>
          </body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;

    let cell_losses = info
        .losses
        .iter()
        .filter(|(label, ..)| label.contains("/td/") || label.contains("/th/"))
        .collect::<Vec<_>>();
    assert!(
        cell_losses.is_empty(),
        "unexpected cell losses: {cell_losses:?}"
    );

    let (jats, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    for expected in [
        "Header",
        "First paragraph",
        "Second paragraph",
        "Plain ",
        "inline",
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// Unsupported citation children and attributes should be reported against the
/// citation, not the surrounding `<ref>`
#[tokio::test]
async fn citation_losses_use_citation_paths() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <ref-list>
              <ref id="ref1">
                <element-citation publication-type="journal" publisher-type="commercial">
                  <person-group person-group-type="author">
                    <name><surname>Smith</surname><given-names>Alex</given-names></name>
                    <etal/>
                  </person-group>
                  <article-title>Referenced work</article-title>
                  <source>A Journal</source>
                  <year>2024</year>
                  <publisher-name>A Publisher</publisher-name>
                  <comment>An editorial comment</comment>
                  <pub-id pub-id-type="doi">10.1234/reference</pub-id>
                  <pub-id pub-id-type="pmid">12345678</pub-id>
                </element-citation>
              </ref>
            </ref-list>
          </back>
        </article>
    "#;

    let (.., info) = JatsCodec.from_str(source, None).await?;
    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();

    let citation = "//article/back/ref-list/ref/element-citation";
    for expected in [
        format!("{citation}/@publisher-type"),
        format!("{citation}/publisher-name"),
        format!("{citation}/comment"),
        format!("{citation}/person-group/etal"),
        format!("{citation}/pub-id[@pub-id-type='pmid']"),
    ] {
        assert!(
            labels.contains(&expected.as_str()),
            "missing {expected} in {labels:?}"
        );
    }

    // Nothing should be attributed to the <ref> itself
    assert!(
        !labels.contains(&"//article/back/ref-list/ref"),
        "citation details reported against the surrounding ref: {labels:?}"
    );

    Ok(())
}

/// Populated properties that JATS encoding does not emit should each produce a
/// loss
#[tokio::test]
async fn encoding_reports_unsupported_article_properties() -> Result<()> {
    let mut article = Article::new(vec![p([t("Body")])]);
    article.options.identifiers = Some(vec![PropertyValueOrString::String("pmid:12345678".into())]);
    article.options.licenses = Some(vec![CreativeWorkVariantOrString::String(
        "https://creativecommons.org/licenses/by/4.0/".into(),
    )]);
    article.options.date_modified = Some(DateTime::new("2025-06-07".into()));
    article.options.editors = Some(vec![Person {
        family_names: Some(vec!["Roe".into()]),
        ..Default::default()
    }]);
    article.options.genre = Some(vec!["Research Article".into()]);
    article.options.description = Some("A description".into());

    let (.., info) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    for expected in [
        "Article.identifiers",
        "Article.licenses",
        "Article.dateModified",
        "Article.editors",
        "Article.genre",
        "Article.description",
    ] {
        assert!(
            labels.contains(&expected),
            "missing {expected} in {labels:?}"
        );
    }

    Ok(())
}

/// Reference container metadata that is not emitted should be reported against
/// the container
#[tokio::test]
async fn encoding_reports_unsupported_reference_properties() -> Result<()> {
    let mut container = Reference::new();
    container.work_type = Some(CreativeWorkType::Periodical);
    container.title = Some(vec![t("A Journal")]);
    container.options.volume_number = Some(IntegerOrString::Integer(12));
    container.options.issue_number = Some(IntegerOrString::Integer(3));

    let mut reference = Reference::new();
    reference.title = Some(vec![t("Referenced work")]);
    reference.date = Some(Date::new("2024".into()));
    reference.is_part_of = Some(Box::new(container));
    reference.options.identifiers =
        Some(vec![PropertyValueOrString::String("pmid:12345678".into())]);

    let mut article = Article::new(Vec::new());
    article.references = Some(vec![reference]);

    let (.., info) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    for expected in [
        "Reference.identifiers",
        "Reference.isPartOf.volumeNumber",
        "Reference.isPartOf.issueNumber",
    ] {
        assert!(
            labels.contains(&expected),
            "missing {expected} in {labels:?}"
        );
    }

    Ok(())
}

/// Blocks nested in a `<p>` should split it rather than being dropped
///
/// JATS allows lists, boxed text and quotes inside a paragraph. Previously only
/// figures, tables, formulas and supplementary material were recognized, so
/// everything else, including a 2,339 character list in one of the examples,
/// was silently discarded.
#[tokio::test]
async fn blocks_nested_in_paragraphs_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>Before<list list-type="order"><list-item><p>Item</p></list-item></list>After</p>
          </body>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    assert_eq!(article.content.len(), 3);
    assert!(matches!(article.content[0], Block::Paragraph(..)));
    assert!(matches!(article.content[1], Block::List(..)));
    assert!(matches!(article.content[2], Block::Paragraph(..)));

    let (jats, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(jats.contains("<list"), "list not encoded in {jats}");
    assert!(jats.contains("Item"), "list content not encoded in {jats}");

    Ok(())
}

/// Grouped footnotes should survive as notes or, when section like, as a section
#[tokio::test]
async fn grouped_footnotes_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <fn-group>
              <fn id="fn1" fn-type="custom" custom-type="endnote">
                <label>1</label>
                <p>An endnote</p>
              </fn>
            </fn-group>
            <fn-group content-type="competing-interest">
              <fn id="conf1" fn-type="COI-statement"><p>No competing interests</p></fn>
            </fn-group>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    let Some(Block::Paragraph(paragraph)) = article.content.first() else {
        bail!("expected the ungrouped notes to be a paragraph")
    };
    let Some(Inline::Note(note)) = paragraph.content.first() else {
        bail!("expected a note")
    };
    assert_eq!(note.id.as_deref(), Some("fn1"));
    assert_eq!(note.note_type, NoteType::Endnote);
    assert_eq!(to_text(&note.content).trim(), "An endnote");

    let Some(Block::Section(section)) = article.content.get(1) else {
        bail!("expected the competing interests to be a section")
    };
    assert_eq!(section.section_type, Some(SectionType::CompetingInterests));
    assert_eq!(to_text(&section.content).trim(), "No competing interests");

    let (jats, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(jats.contains(r#"<fn fn-type="custom" id="fn1""#), "{jats}");
    assert!(jats.contains("An endnote"), "{jats}");
    assert!(jats.contains("No competing interests"), "{jats}");

    Ok(())
}

/// Additional abstracts should be kept with their type rather than discarded
#[tokio::test]
async fn additional_abstracts_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <front>
            <article-meta>
              <abstract><p>The abstract</p></abstract>
              <abstract id="abs2" abstract-type="graphical"><p>The graphical abstract</p></abstract>
            </article-meta>
          </front>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    assert_eq!(to_text(&article.r#abstract).trim(), "The abstract");

    let Some(CreativeWorkVariant::Article(part)) = article.options.parts.iter().flatten().next()
    else {
        bail!("expected the additional abstract to be a part")
    };
    assert_eq!(part.id.as_deref(), Some("abs2"));
    assert_eq!(
        part.options.genre.as_deref(),
        Some(["graphical".to_string()].as_slice())
    );
    assert_eq!(to_text(&part.r#abstract).trim(), "The graphical abstract");

    let (jats, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(
        jats.contains(r#"<abstract id="abs2" abstract-type="graphical">"#),
        "{jats}"
    );
    assert!(jats.contains("The graphical abstract"), "{jats}");

    Ok(())
}

/// Sub-articles, such as eLife reviews and assessments, should round-trip
#[tokio::test]
async fn sub_articles_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body><p>The article</p></body>
          <sub-article article-type="referee-report" id="sa1">
            <front-stub>
              <article-id pub-id-type="doi">10.7554/eLife.1.sa1</article-id>
              <title-group><article-title>Reviewer #1</article-title></title-group>
            </front-stub>
            <body><p>The review</p></body>
          </sub-article>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    let Some(CreativeWorkVariant::Article(part)) = article.options.parts.iter().flatten().next()
    else {
        bail!("expected the sub-article to be a part")
    };
    assert_eq!(part.id.as_deref(), Some("sa1"));
    assert_eq!(
        part.options.genre.as_deref(),
        Some(["referee-report".to_string()].as_slice())
    );
    assert_eq!(part.doi.as_deref(), Some("10.7554/eLife.1.sa1"));
    assert_eq!(to_text(&part.title).trim(), "Reviewer #1");
    assert_eq!(to_text(&part.content).trim(), "The review");

    let (jats, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(
        jats.contains(r#"<sub-article id="sa1" article-type="referee-report">"#),
        "{jats}"
    );
    assert!(jats.contains("<front-stub>"), "{jats}");
    assert!(jats.contains("The review"), "{jats}");

    // The nested work must survive another cycle
    let (node, ..) = JatsCodec.from_str(&jats, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };
    assert_eq!(article.options.parts.iter().flatten().count(), 1);

    Ok(())
}
