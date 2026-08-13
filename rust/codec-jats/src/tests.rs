use pretty_assertions::assert_eq;
use stencila_codec::Losses;
use stencila_codec::eyre::bail;
use stencila_codec::stencila_schema::{
    Article, Author, Block, CompilationDigest, CreativeWorkType, CreativeWorkVariant,
    CreativeWorkVariantOrString, Date, DateTime, Figure, FigureOptions, Inline, IntegerOrString,
    Link, Node, NoteType, Organization, Person, PersonOrOrganization, Primitive, PropertyValue,
    PropertyValueOptions, PropertyValueOrString, Reference, SectionType,
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

/// Contributors, their affiliations and the notes about them should survive a
/// round trip wherever JATS writes them
#[tokio::test]
async fn contributors_and_affiliations_roundtrip() -> Result<()> {
    let jats = r#"
        <article>
          <front>
            <article-meta>
              <contrib-group>
                <contrib contrib-type="author" equal-contrib="yes">
                  <name><surname>Doe</surname><given-names>Jane</given-names></name>
                  <contrib-id contrib-id-type="orcid">https://orcid.org/0000-0002-1825-0097</contrib-id>
                  <xref ref-type="aff" rid="aff1">1</xref>
                  <xref ref-type="fn" rid="equal1"/>
                </contrib>
                <contrib contrib-type="author">
                  <collab>The Example Consortium</collab>
                </contrib>
              </contrib-group>
              <contrib-group content-type="editor">
                <contrib contrib-type="editor">
                  <name><surname>Roe</surname><given-names>Richard</given-names></name>
                  <role>Reviewing Editor</role>
                </contrib>
              </contrib-group>
              <aff id="aff1">
                <institution-wrap>
                  <institution>Example University</institution>
                  <institution-id institution-id-type="ror">https://ror.org/03yrm5c26</institution-id>
                  <institution-id institution-id-type="isni">0000000123456789</institution-id>
                </institution-wrap>
                <country>New Zealand</country>
              </aff>
              <author-notes>
                <fn fn-type="con" id="equal1"><label>&#8224;</label><p>These authors contributed equally</p></fn>
              </author-notes>
            </article-meta>
          </front>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    // A `<collab>` names a group rather than a person
    let authors = article.authors.as_ref().expect("should have authors");
    assert!(matches!(authors[1], Author::Organization(..)));

    // An affiliation that stands on its own is still resolved by the
    // contributor that refers to it
    let Author::Person(person) = &authors[0] else {
        bail!("expected a person author")
    };
    let affiliation = &person.affiliations.as_ref().expect("affiliated")[0];
    assert_eq!(affiliation.name.as_deref(), Some("Example University"));
    assert_eq!(affiliation.ror.as_deref(), Some("03yrm5c26"));

    // The role of an editor is kept as their job title
    let editors = article
        .options
        .editors
        .as_ref()
        .expect("should have editors");
    assert_eq!(
        editors[0].options.job_title.as_deref(),
        Some("Reviewing Editor")
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
        r#"<collab>The Example Consortium</collab>"#,
        r#"<contrib contrib-type="editor">"#,
        r#"<role>Reviewing Editor</role>"#,
        r#"<institution-id institution-id-type="isni">0000000123456789</institution-id>"#,
        r#"<fn id="equal1" fn-type="con"><label>†</label><p>These authors contributed equally</p></fn>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// Award identifiers, funding sources and recipients should survive a round trip
#[tokio::test]
async fn funding_and_awards_roundtrip() -> Result<()> {
    let jats = r#"
        <article>
          <front>
            <article-meta>
              <funding-group>
                <award-group id="fund1">
                  <funding-source>
                    <institution-wrap>
                      <institution-id institution-id-type="FundRef">http://dx.doi.org/10.13039/100000002</institution-id>
                      <institution>National Institutes of Health</institution>
                    </institution-wrap>
                  </funding-source>
                  <award-id>R35 GM127029</award-id>
                  <principal-award-recipient>
                    <name><surname>Doe</surname><given-names>Jane</given-names></name>
                  </principal-award-recipient>
                </award-group>
              </funding-group>
            </article-meta>
          </front>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    let funders = article
        .options
        .funders
        .as_ref()
        .expect("should have funders");
    assert_eq!(funders.len(), 1);
    let grants = article
        .options
        .funded_by
        .as_ref()
        .expect("should have grants");
    assert_eq!(grants.len(), 1);

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
        r#"<award-group id="fund1">"#,
        r#"<institution>National Institutes of Health</institution>"#,
        r#"<institution-id institution-id-type="FundRef">http://dx.doi.org/10.13039/100000002</institution-id>"#,
        r#"<award-id>R35 GM127029</award-id>"#,
        r#"<principal-award-recipient><name><surname>Doe</surname><given-names>Jane</given-names></name></principal-award-recipient>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// MathML should be emitted as namespaced XML and mean the same after decoding
/// it again
#[tokio::test]
async fn mathml_roundtrip() -> Result<()> {
    let jats = r#"
        <article xmlns:mml="http://www.w3.org/1998/Math/MathML">
          <body>
            <p>Because <inline-formula id="if1"><mml:math id="m1"><mml:mi>x</mml:mi></mml:math></inline-formula> holds.</p>
            <p>
              <disp-formula id="df1">
                <label>(1)</label>
                <alternatives>
                  <tex-math>E = mc^2</tex-math>
                  <mml:math id="m2" display="block"><mml:mi>E</mml:mi></mml:math>
                </alternatives>
              </disp-formula>
            </p>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let (encoded, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // The MathML is real elements rather than an attribute that only Stencila
    // can read, and it carries the namespace that makes it MathML
    assert!(
        encoded.contains(
            r#"<mml:math xmlns:mml="http://www.w3.org/1998/Math/MathML" id="m1"><mml:mi>x</mml:mi></mml:math>"#
        ),
        "MathML not emitted as namespaced XML: {encoded}"
    );
    assert!(
        !encoded.contains("code=\"&lt;"),
        "math source emitted as an escaped attribute: {encoded}"
    );

    // Where a source states the math both ways, both survive
    assert!(encoded.contains("<tex-math>E = mc^2</tex-math>"));
    assert!(encoded.contains(r#"<disp-formula id="df1">"#));
    assert!(encoded.contains("<label>(1)</label>"));

    // Decoding the emitted JATS gives the same math again
    let (again, ..) = JatsCodec.from_str(&encoded, None).await?;
    assert_eq!(again, node);

    Ok(())
}

/// A formula stated only as an image should keep that image
#[tokio::test]
async fn formula_images_are_preserved() -> Result<()> {
    let jats = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body>
            <p>
              <disp-formula id="eqn1"><graphic xlink:href="eqn1.gif"/></disp-formula>
            </p>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let (encoded, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    assert!(
        encoded.contains(r#"<graphic xlink:href="eqn1.gif"></graphic>"#),
        "formula image dropped: {encoded}"
    );

    Ok(())
}

/// Media metadata, and the JATS element that a resource belongs in, should
/// survive a round trip
#[tokio::test]
async fn media_and_supplements_roundtrip() -> Result<()> {
    let jats = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body>
            <sec>
              <fig id="fig1">
                <label>Figure 1</label>
                <graphic xlink:href="fig1.tif" mimetype="image" mime-subtype="tiff"/>
              </fig>
              <supplementary-material id="sup1">
                <label>Video 1</label>
                <media xlink:href="movie1.mp4" mimetype="video" mime-subtype="mp4"/>
              </supplementary-material>
            </sec>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let (encoded, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // An image that stands on its own is a `<graphic>`, not an
    // `<inline-graphic>`, and keeps the type of file that it is
    assert!(
        encoded.contains(
            r#"<graphic xlink:href="fig1.tif" mimetype="image" mime-subtype="tiff"></graphic>"#
        ),
        "figure image not emitted as a typed graphic: {encoded}"
    );

    // A supplement points at its resource through a `<media>`, which is where
    // JATS consumers look for it
    assert!(
        encoded.contains(r#"<media xlink:href="movie1.mp4"></media>"#),
        "supplement target not emitted as media: {encoded}"
    );
    assert!(
        !encoded.contains(r#"<supplementary-material id="sup1" href="#),
        "supplement target emitted as an attribute: {encoded}"
    );

    // The kind of work a supplement is, which is inferred rather than stated,
    // is inferred the same way again
    let (again, ..) = JatsCodec.from_str(&encoded, None).await?;
    assert_eq!(again, node);

    Ok(())
}

/// Text within inline elements that the schema has no node for should be kept,
/// and table cell alignment should survive a round trip
#[tokio::test]
async fn inline_and_table_cell_fallbacks() -> Result<()> {
    let jats = r#"
        <article>
          <body>
            <sec>
              <p>The <named-content content-type="gene">BRCA1</named-content> gene, see <email>jane@example.org</email>.</p>
              <table-wrap id="t1">
                <table>
                  <tbody>
                    <tr>
                      <td align="char" char="." valign="top">1.5<break/>2.5</td>
                      <td align="center">middle</td>
                    </tr>
                  </tbody>
                </table>
              </table-wrap>
            </sec>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let (encoded, ..) = JatsCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // The wrapper is reported as lost, but what it wrapped is still readable
    assert!(
        encoded.contains("The BRCA1 gene"),
        "text of an unsupported wrapper dropped: {encoded}"
    );
    // A line break separates the words on either side of it
    assert!(
        encoded.contains("1.5 2.5"),
        "line break ran two values together: {encoded}"
    );
    // An email address is emitted as the element JATS has for one
    assert!(
        encoded.contains("<email>jane@example.org</email>"),
        "email not emitted as an email: {encoded}"
    );
    // Alignment is stated by JATS with values that the schema names differently
    assert!(
        encoded.contains(r#"<td align="char" char="." valign="top">"#),
        "cell alignment dropped: {encoded}"
    );
    assert!(
        encoded.contains(r#"<td align="center">"#),
        "cell alignment dropped: {encoded}"
    );

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

    // The structured fields of the reference and its rendering as text are
    // emitted as alternatives to each other, rather than one within the other
    let citation = root
        .descendants()
        .find(|node| node.has_tag_name("element-citation"))
        .ok_or_else(|| stencila_codec::eyre::eyre!("missing element-citation"))?;
    assert_eq!(
        citation.parent().map(|parent| parent.tag_name().name()),
        Some("citation-alternatives")
    );
    assert_eq!(citation.attribute("publication-type"), Some("journal"));
    assert_eq!(
        citation
            .descendants()
            .find(|node| node.has_tag_name("pub-id"))
            .and_then(|node| node.text()),
        Some("10.1234/example")
    );
    let mixed = root
        .descendants()
        .find(|node| node.has_tag_name("mixed-citation"))
        .ok_or_else(|| stencila_codec::eyre::eyre!("missing mixed-citation"))?;
    assert_eq!(mixed.text(), Some("Doe (2024). Referenced work."));
    assert!(
        !citation
            .descendants()
            .any(|node| node.is_text()
                && node.text().is_some_and(|text| text.contains("Doe (2024)"))),
        "citation text duplicated"
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
        format!("{citation}/comment"),
        format!("{citation}/person-group/etal"),
    ] {
        assert!(
            labels.contains(&expected.as_str()),
            "missing {expected} in {labels:?}"
        );
    }

    // The publisher and the non-DOI identifier are now preserved, so should not
    // be reported as lost
    for unexpected in [
        format!("{citation}/publisher-name"),
        format!("{citation}/pub-id[@pub-id-type='pmid']"),
    ] {
        assert!(
            !labels.contains(&unexpected.as_str()),
            "unexpected {unexpected} in {labels:?}"
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
    article.options.url = Some("https://example.org/article".into());
    article.options.date_modified = Some(DateTime::new("2025-06-07".into()));
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
        "Article.url",
        "Article.dateModified",
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

    container.doi = Some("10.1234/journal".into());

    let mut reference = Reference::new();
    reference.title = Some(vec![t("Referenced work")]);
    reference.date = Some(Date::new("2024".into()));
    reference.is_part_of = Some(Box::new(container));
    reference.options.identifiers =
        Some(vec![PropertyValueOrString::String("pmid:12345678".into())]);

    let mut article = Article::new(Vec::new());
    article.references = Some(vec![reference]);

    let (jats, info) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    // Volume and issue belong to the container but JATS spells them within the
    // citation, so they are emitted rather than reported as lost
    assert!(
        jats.contains("<volume>12</volume>"),
        "missing volume: {jats}"
    );
    assert!(jats.contains("<issue>3</issue>"), "missing issue: {jats}");
    assert!(
        jats.contains("<pub-id>pmid:12345678</pub-id>"),
        "missing identifier: {jats}"
    );

    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    assert!(
        labels.contains(&"Reference.isPartOf.doi"),
        "missing Reference.isPartOf.doi in {labels:?}"
    );
    for unexpected in [
        "Reference.identifiers",
        "Reference.isPartOf.volumeNumber",
        "Reference.isPartOf.issueNumber",
    ] {
        assert!(
            !labels.contains(&unexpected),
            "unexpected {unexpected} in {labels:?}"
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

/// Article, journal, issue, license and resource metadata should survive a
/// round trip
#[tokio::test]
async fn publication_metadata_roundtrip() -> Result<()> {
    let jats = r#"
        <article>
          <front>
            <journal-meta>
              <journal-id journal-id-type="nlm-ta">Example J</journal-id>
              <journal-id journal-id-type="doi">10.1111/(ISSN)1234-5678</journal-id>
              <journal-title-group>
                <journal-title>The Example Journal</journal-title>
                <abbrev-journal-title abbrev-type="publisher">Example J</abbrev-journal-title>
              </journal-title-group>
              <issn pub-type="ppub">1234-5678</issn>
              <issn publication-format="electronic">8765-4321</issn>
            </journal-meta>
            <article-meta>
              <article-id pub-id-type="doi">10.1234/example</article-id>
              <article-id pub-id-type="pmid">12345678</article-id>
              <article-id pub-id-type="other" specific-use="slug">example</article-id>
              <article-categories>
                <subj-group subj-group-type="Discipline">
                  <subject>Biology</subject>
                  <subj-group>
                    <subject>Genetics</subject>
                  </subj-group>
                </subj-group>
              </article-categories>
              <title-group><article-title>An example</article-title></title-group>
              <pub-date pub-type="collection"><year>2025</year></pub-date>
              <pub-date pub-type="epub"><day>4</day><month>3</month><year>2025</year></pub-date>
              <volume>12</volume>
              <issue>3</issue>
              <issue-id pub-id-type="doi">10.1234/example.issue-3</issue-id>
              <issue-title>A special issue</issue-title>
              <elocation-id>e0012345</elocation-id>
              <permissions>
                <copyright-statement>© 2025 The Authors</copyright-statement>
                <copyright-year>2025</copyright-year>
                <license xlink:href="https://creativecommons.org/licenses/by/4.0/"
                         xmlns:xlink="http://www.w3.org/1999/xlink">
                  <license-p>Reuse is permitted with attribution.</license-p>
                </license>
              </permissions>
              <self-uri content-type="pdf" xlink:href="example.pdf"
                        xmlns:xlink="http://www.w3.org/1999/xlink" />
            </article-meta>
          </front>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };

    // The most specific publication date wins, rather than the last one
    assert_eq!(
        article
            .date_published
            .as_ref()
            .map(|date| date.value.clone()),
        Some("2025-03-04".to_string())
    );
    assert_eq!(article.doi.as_deref(), Some("10.1234/example"));
    assert_eq!(
        article.options.licenses,
        Some(vec![CreativeWorkVariantOrString::String(
            "https://creativecommons.org/licenses/by/4.0/".into()
        )])
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
        r#"<journal-id journal-id-type="doi">10.1111/(ISSN)1234-5678</journal-id>"#,
        r#"<journal-id journal-id-type="nlm-ta">Example J</journal-id>"#,
        "<journal-title>The Example Journal</journal-title>",
        r#"<abbrev-journal-title>Example J</abbrev-journal-title>"#,
        r#"<issn pub-type="ppub">1234-5678</issn>"#,
        r#"<issn pub-type="epub">8765-4321</issn>"#,
        r#"<article-id pub-id-type="pmid">12345678</article-id>"#,
        r#"<article-id pub-id-type="other" specific-use="slug">example</article-id>"#,
        // The subject hierarchy is preserved, not flattened
        r#"<subj-group subj-group-type="Discipline"><subject>Biology</subject><subj-group><subject>Genetics</subject></subj-group></subj-group>"#,
        r#"<pub-date pub-type="collection" iso-8601-date="2025">"#,
        r#"<pub-date pub-type="epub" iso-8601-date="2025-03-04">"#,
        r#"<issue-id pub-id-type="doi">10.1234/example.issue-3</issue-id>"#,
        "<issue-title>A special issue</issue-title>",
        // An electronic location is not confused with a page range
        "<elocation-id>e0012345</elocation-id>",
        "<copyright-statement>© 2025 The Authors</copyright-statement>",
        r#"<license xlink:href="https://creativecommons.org/licenses/by/4.0/">"#,
        "<license-p>Reuse is permitted with attribution.</license-p>",
        r#"<self-uri content-type="pdf" xlink:href="example.pdf">"#,
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// Links into a publishing system's own file system are not portable, so
/// should be filtered out and reported
#[tokio::test]
async fn non_portable_self_uris_are_filtered() -> Result<()> {
    let jats = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <front>
            <article-meta>
              <self-uri content-type="pdf" xlink:href="example.pdf" />
              <self-uri content-type="pdf" xlink:href="file:/content/journal/vol1/example.pdf" />
            </article-meta>
          </front>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(jats, None).await?;
    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    assert!(
        labels.contains(&"//article/front/article-meta/self-uri/@href"),
        "{labels:?}"
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
    assert!(jats.contains("example.pdf"), "{jats}");
    assert!(!jats.contains("file:/content"), "{jats}");

    Ok(())
}

/// A journal article citation should round-trip its container metadata
///
/// The volume and issue of the journal, and the pages of the article within it,
/// are spelt as siblings in JATS but belong to different works in the schema.
#[tokio::test]
async fn reference_container_metadata_roundtrip() -> Result<()> {
    let source = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <back>
            <ref-list>
              <ref id="ref1">
                <element-citation publication-type="journal">
                  <person-group person-group-type="author">
                    <name><surname>Smith</surname><given-names>Alex</given-names></name>
                  </person-group>
                  <article-title>Referenced work</article-title>
                  <source>A Journal</source>
                  <year>2024</year>
                  <volume>12</volume>
                  <issue>3</issue>
                  <fpage>100</fpage>
                  <lpage>110</lpage>
                  <pub-id pub-id-type="doi">10.1234/reference</pub-id>
                  <pub-id pub-id-type="pmid">12345678</pub-id>
                  <ext-link ext-link-type="uri" xlink:href="https://example.org/work">https://example.org/work</ext-link>
                </element-citation>
              </ref>
            </ref-list>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };
    let Some(reference) = article.references.iter().flatten().next() else {
        bail!("expected a reference")
    };

    assert_eq!(reference.id.as_deref(), Some("ref1"));
    assert_eq!(reference.doi.as_deref(), Some("10.1234/reference"));
    assert_eq!(reference.url.as_deref(), Some("https://example.org/work"));
    assert_eq!(
        reference.options.page_start,
        Some(IntegerOrString::Integer(100))
    );

    // The volume and issue are of the journal, not of the article
    let Some(container) = &reference.is_part_of else {
        bail!("expected a container")
    };
    assert_eq!(container.work_type, Some(CreativeWorkType::Periodical));
    assert_eq!(
        container.options.volume_number,
        Some(IntegerOrString::Integer(12))
    );
    assert_eq!(reference.options.volume_number, None);

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
        "<source>A Journal</source>",
        "<volume>12</volume>",
        "<issue>3</issue>",
        "<fpage>100</fpage>",
        "<lpage>110</lpage>",
        r#"<pub-id pub-id-type="pmid">12345678</pub-id>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// Text parsing should fill in missing fields without replacing decoded ones
#[tokio::test]
async fn reference_text_only_fills_missing_fields() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <ref-list>
              <ref id="ref1">
                <mixed-citation publication-type="journal">Jones, B. (1999) A quite different title. Some Other Journal 5, 1-2.
                  <article-title>The decoded title</article-title>
                  <year>2024</year>
                </mixed-citation>
              </ref>
            </ref-list>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };
    let Some(reference) = article.references.iter().flatten().next() else {
        bail!("expected a reference")
    };

    // The elements win over the text they are mixed with
    assert_eq!(
        reference.title.as_ref().map(to_text),
        Some("The decoded title".to_string())
    );
    assert_eq!(
        reference.date.as_ref().map(|date| date.value.clone()),
        Some("2024".into())
    );

    // The authors, which no element supplied, come from the text
    assert_eq!(reference.authors.iter().flatten().count(), 1);

    // With a title, the reference can be rendered from its fields, so the raw
    // text is not kept as well
    assert_eq!(reference.options.text, None);

    Ok(())
}

/// A citation whose fields can not all be decoded should keep its raw text as
/// an alternative to, not as well as, those that can
#[tokio::test]
async fn reference_keeps_raw_text_as_an_alternative() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <ref-list>
              <ref id="ref1">
                <citation-alternatives>
                  <element-citation publication-type="journal">
                    <person-group person-group-type="author">
                      <name><surname>Smith</surname><given-names>Alex</given-names></name>
                    </person-group>
                    <year>2024</year>
                  </element-citation>
                  <mixed-citation>Smith, A. An untitled thing worth keeping. (2024).</mixed-citation>
                </citation-alternatives>
              </ref>
            </ref-list>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };
    let Some(reference) = article.references.iter().flatten().next() else {
        bail!("expected a reference")
    };
    assert!(
        reference
            .options
            .text
            .as_deref()
            .is_some_and(|text| text.contains("An untitled thing worth keeping")),
        "raw citation text not kept: {:?}",
        reference.options.text
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
    assert!(jats.contains("<citation-alternatives>"), "{jats}");
    assert_eq!(
        jats.matches("Smith").count(),
        2,
        "author duplicated between the alternatives: {jats}"
    );

    Ok(())
}

/// A book chapter citation should attach editors and the publisher to the book
#[tokio::test]
async fn reference_chapter_editors_and_publisher() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <ref-list>
              <ref id="ref1">
                <element-citation publication-type="book">
                  <person-group person-group-type="author">
                    <name><surname>Smith</surname><given-names>Alex</given-names></name>
                  </person-group>
                  <person-group person-group-type="editor">
                    <name><surname>Roe</surname><given-names>Jo</given-names></name>
                  </person-group>
                  <chapter-title>A chapter</chapter-title>
                  <source>A Book</source>
                  <edition>3</edition>
                  <publisher-name>A Publisher</publisher-name>
                  <publisher-loc>A Place</publisher-loc>
                  <year>2024</year>
                </element-citation>
              </ref>
            </ref-list>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = &node else {
        bail!("expected an article")
    };
    let Some(reference) = article.references.iter().flatten().next() else {
        bail!("expected a reference")
    };

    assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
    let Some(container) = &reference.is_part_of else {
        bail!("expected a container")
    };
    // The editors edited the book, and the publisher published it
    assert_eq!(container.options.editors.iter().flatten().count(), 1);
    assert_eq!(reference.options.editors, None);
    assert!(container.options.publisher.is_some());

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
        r#"<person-group person-group-type="editor">"#,
        "<chapter-title>A chapter</chapter-title>",
        "<source>A Book</source>",
        "<edition>3</edition>",
        "<publisher-loc>A Place</publisher-loc>",
        "<publisher-name>A Publisher</publisher-name>",
    ] {
        assert!(jats.contains(expected), "missing {expected} in {jats}");
    }

    Ok(())
}

/// A mixed alternative should fill every missing field that its text parser can recover
#[tokio::test]
async fn reference_alternative_fills_metadata_beyond_authors_and_title() -> Result<()> {
    let source = r#"
        <article>
          <back>
            <ref-list>
              <ref id="ref1">
                <citation-alternatives>
                  <element-citation publication-type="journal">
                    <person-group person-group-type="author">
                      <name><surname>Jones</surname><given-names>Betty</given-names></name>
                    </person-group>
                    <article-title>A parsed title</article-title>
                  </element-citation>
                  <mixed-citation>Jones, B. (1999). A parsed title. A Journal, 5, 1-2. https://doi.org/10.1234/alternative</mixed-citation>
                </citation-alternatives>
              </ref>
            </ref-list>
          </back>
        </article>"#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    let Some(reference) = article.references.iter().flatten().next() else {
        bail!("expected a reference")
    };

    assert_eq!(
        reference.date.as_ref().map(|date| date.value.as_str()),
        Some("1999")
    );
    assert_eq!(reference.doi.as_deref(), Some("10.1234/alternative"));

    Ok(())
}

/// Identifier qualifiers should survive encoding and decoding
#[tokio::test]
async fn reference_identifier_qualifier_roundtrip() -> Result<()> {
    let mut identifier = PropertyValue::new(Primitive::String("12345678".into()));
    identifier.property_id = Some("pmid".into());
    identifier.options = Box::new(PropertyValueOptions {
        name: Some("versioned".into()),
        ..Default::default()
    });

    let mut reference = Reference::new();
    reference.title = Some(vec![t("Referenced work")]);
    reference.options.identifiers = Some(vec![PropertyValueOrString::PropertyValue(identifier)]);
    let mut article = Article::new(Vec::new());
    article.references = Some(vec![reference]);

    let (jats, ..) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(
        jats.contains(r#"<pub-id pub-id-type="pmid" specific-use="versioned">12345678</pub-id>"#),
        "{jats}"
    );

    let (node, info) = JatsCodec.from_str(&jats, None).await?;
    let losses = info.losses.iter().collect::<Vec<_>>();
    assert!(losses.is_empty(), "{losses:?}");
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    let Some(PropertyValueOrString::PropertyValue(identifier)) = article
        .references
        .iter()
        .flatten()
        .next()
        .and_then(|reference| reference.options.identifiers.as_ref())
        .and_then(|identifiers| identifiers.first())
    else {
        bail!("expected a property-value identifier")
    };
    assert_eq!(identifier.options.name.as_deref(), Some("versioned"));

    Ok(())
}

/// Flat citation fields should prefer the schema level that decoding reconstructs
#[tokio::test]
async fn reference_container_conflicts_are_reported_without_overwriting_container() -> Result<()> {
    let mut container = Reference::new();
    container.work_type = Some(CreativeWorkType::Book);
    container.title = Some(vec![t("Container book")]);
    container.options.editors = Some(vec![Person {
        family_names: Some(vec!["ContainerEditor".into()]),
        ..Default::default()
    }]);
    container.options.publisher = Some(PersonOrOrganization::Organization(Organization {
        name: Some("Container Publisher".into()),
        ..Default::default()
    }));
    container.options.volume_number = Some(IntegerOrString::Integer(2));
    container.options.issue_number = Some(IntegerOrString::Integer(3));

    let mut reference = Reference::new();
    reference.work_type = Some(CreativeWorkType::Chapter);
    reference.title = Some(vec![t("A chapter")]);
    reference.is_part_of = Some(Box::new(container));
    reference.options.editors = Some(vec![Person {
        family_names: Some(vec!["ReferenceEditor".into()]),
        ..Default::default()
    }]);
    reference.options.publisher = Some(PersonOrOrganization::Organization(Organization {
        name: Some("Reference Publisher".into()),
        ..Default::default()
    }));
    reference.options.volume_number = Some(IntegerOrString::Integer(8));
    reference.options.issue_number = Some(IntegerOrString::Integer(9));

    let mut article = Article::new(Vec::new());
    article.references = Some(vec![reference]);
    let (jats, info) = JatsCodec
        .to_string(
            &Node::Article(article),
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    for expected in [
        "<surname>ContainerEditor</surname>",
        "<publisher-name>Container Publisher</publisher-name>",
        "<volume>2</volume>",
        "<issue>3</issue>",
    ] {
        assert!(jats.contains(expected), "missing {expected}: {jats}");
    }
    for unexpected in [
        "ReferenceEditor",
        "Reference Publisher",
        "<volume>8</volume>",
        "<issue>9</issue>",
    ] {
        assert!(!jats.contains(unexpected), "retained {unexpected}: {jats}");
    }

    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    for expected in [
        "Reference.editors",
        "Reference.publisher",
        "Reference.volumeNumber",
        "Reference.issueNumber",
    ] {
        assert!(labels.contains(&expected), "missing {expected}: {labels:?}");
    }

    Ok(())
}

/// Encode compactly, which is what the cross reference tests inspect
async fn to_compact_jats(node: &Node) -> Result<(String, Losses)> {
    let (jats, info) = JatsCodec
        .to_string(
            node,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    Ok((jats, info.losses))
}

/// Internal links are encoded as `<xref>` with the `ref-type` of their target
#[tokio::test]
async fn internal_links_are_typed_cross_references() -> Result<()> {
    let source = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body>
            <sec id="s1">
              <title>Methods</title>
              <p>
                See <xref ref-type="sec" rid="s1">Methods</xref>,
                <xref ref-type="fig" rid="f1">Figure 1</xref>,
                <xref ref-type="table" rid="t1">Table 1</xref>,
                <xref ref-type="disp-formula" rid="e1">Equation 1</xref>,
                <xref ref-type="supplementary-material" rid="sup1">File S1</xref>,
                <xref ref-type="other" rid="b1">Box 1</xref> and
                <ext-link ext-link-type="uri" xlink:href="https://example.org">a site</ext-link>.
              </p>
              <fig id="f1"><caption><p>A figure</p></caption></fig>
              <table-wrap id="t1"><table><tbody><tr><td>Cell</td></tr></tbody></table></table-wrap>
              <disp-formula id="e1"><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi></math></disp-formula>
              <supplementary-material id="sup1"><media xlink:href="s1.csv"/></supplementary-material>
              <boxed-text id="b1"><p>A box</p></boxed-text>
            </sec>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    for expected in [
        r#"<xref ref-type="sec" rid="s1">Methods</xref>"#,
        r#"<xref ref-type="fig" rid="f1">Figure 1</xref>"#,
        r#"<xref ref-type="table" rid="t1">Table 1</xref>"#,
        r#"<xref ref-type="disp-formula" rid="e1">Equation 1</xref>"#,
        r#"<xref ref-type="supplementary-material" rid="sup1">File S1</xref>"#,
        r#"<xref ref-type="other" rid="b1">Box 1</xref>"#,
        r#"<ext-link ext-link-type="uri" xlink:href="https://example.org">a site</ext-link>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected}: {jats}");
    }

    Ok(())
}

/// A `ref-type` that says more than the kind of its target node is retained
#[tokio::test]
async fn refined_cross_reference_types_are_retained() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <table-wrap id="t1">
              <table><tbody><tr><td>Cell<xref ref-type="table-fn" rid="tfn1">*</xref></td></tr></tbody></table>
              <table-wrap-foot><fn id="tfn1"><p>A table footnote</p></fn></table-wrap-foot>
            </table-wrap>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="table-fn" rid="tfn1">*</xref>"#),
        "table footnote reference not retained: {jats}"
    );

    Ok(())
}

/// Advisory titles on cross references survive the round trip
#[tokio::test]
async fn cross_reference_titles_are_preserved() -> Result<()> {
    let source = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body>
            <p><xref ref-type="fig" rid="f1" xlink:title="Open figure">Figure 1</xref></p>
            <fig id="f1"><caption><p>A figure</p></caption></fig>
          </body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="fig" rid="f1" xlink:title="Open figure">Figure 1</xref>"#),
        "cross-reference title not retained: {jats}"
    );
    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    assert!(
        !labels.iter().any(|label| label.ends_with("/@title")),
        "cross-reference title reported as lost: {labels:?}"
    );

    Ok(())
}

/// URI link types survive even when the URI is relative
#[tokio::test]
async fn relative_uri_link_types_are_preserved() -> Result<()> {
    let source = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body><p><ext-link ext-link-type="uri" xlink:href="page.html">next page</ext-link></p></body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(
            r#"<ext-link ext-link-type="uri" xlink:href="page.html">next page</ext-link>"#
        ),
        "relative URI type not retained: {jats}"
    );
    assert!(info.losses.is_empty(), "unexpected decode losses");
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// Publisher-specific external link types remain distinct from HTML relationships
#[tokio::test]
async fn custom_external_link_types_are_preserved() -> Result<()> {
    let source = r#"
        <article xmlns:xlink="http://www.w3.org/1999/xlink">
          <body><p><ext-link ext-link-type="genbank" xlink:href="ABC123">record</ext-link></p></body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<ext-link ext-link-type="genbank" xlink:href="ABC123">record</ext-link>"#),
        "custom external link type was not retained: {jats}"
    );
    assert!(info.losses.is_empty(), "unexpected decode losses");
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// A cross reference to several targets keeps all of them
#[tokio::test]
async fn multiple_cross_reference_targets_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>See <xref ref-type="fig" rid="f1 f2">Figures 1 and 2</xref></p>
            <fig id="f1"><caption><p>First</p></caption></fig>
            <fig id="f2"><caption><p>Second</p></caption></fig>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="fig" rid="f1 f2">Figures 1 and 2</xref>"#),
        "multiple targets not preserved: {jats}"
    );

    Ok(())
}

/// A cross reference spanning unlike target kinds uses the general JATS type
#[tokio::test]
async fn mixed_cross_reference_targets_use_other_type() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>See <xref ref-type="other" rid="f1 t1">these items</xref></p>
            <fig id="f1"><caption><p>A figure</p></caption></fig>
            <table-wrap id="t1"><table><tbody><tr><td>Cell</td></tr></tbody></table></table-wrap>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="other" rid="f1 t1">these items</xref>"#),
        "mixed targets were falsely typed: {jats}"
    );
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// The root article is an addressable cross-reference target
#[tokio::test]
async fn root_article_links_are_preserved() -> Result<()> {
    let source = r#"
        <article id="article1">
          <body><p><xref ref-type="other" rid="article1">this article</xref></p></body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="other" rid="article1">this article</xref>"#),
        "root article link was not retained: {jats}"
    );
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// Scheme cross references are retained as links to figures
#[tokio::test]
async fn scheme_cross_references_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p><xref ref-type="scheme" rid="f1">Scheme 1</xref></p>
            <fig id="f1"><caption><p>A scheme</p></caption></fig>
          </body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="scheme" rid="f1">Scheme 1</xref>"#),
        "scheme cross-reference was not retained: {jats}"
    );
    assert!(info.losses.is_empty(), "unexpected decode losses");
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// Publisher-specific cross-reference types survive when their target resolves
#[tokio::test]
async fn custom_cross_reference_types_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p><xref ref-type="illustration" rid="f1">Illustration 1</xref></p>
            <fig id="f1"><caption><p>An illustration</p></caption></fig>
          </body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="illustration" rid="f1">Illustration 1</xref>"#),
        "custom cross-reference type was not retained: {jats}"
    );
    assert!(info.losses.is_empty(), "unexpected decode losses");
    assert!(losses.is_empty(), "unexpected encode losses");

    Ok(())
}

/// A cross reference whose target is not in the document is not emitted
#[tokio::test]
async fn unresolved_cross_references_are_reported_not_emitted() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>See <xref ref-type="fig" rid="missing">Figure 9</xref></p>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        !jats.contains("rid=\"missing\""),
        "dangling reference emitted: {jats}"
    );
    assert!(
        jats.contains("<p>See Figure 9</p>"),
        "reference text not retained: {jats}"
    );

    let labels = losses.iter().map(|(label, ..)| label).collect::<Vec<_>>();
    assert!(
        labels.contains(&"Link.target"),
        "dropped target not reported: {labels:?}"
    );

    Ok(())
}

/// Relationships that cannot be represented on an internal cross reference are losses
#[tokio::test]
async fn unsupported_internal_link_relationships_are_reported() -> Result<()> {
    let node = art([
        p([Inline::Link(Link {
            content: vec![t("Figure 1")],
            target: "#f1".to_string(),
            rel: Some("nofollow".to_string()),
            ..Default::default()
        })]),
        Block::Figure(Figure {
            id: Some("f1".to_string()),
            ..Default::default()
        }),
    ]);

    let (jats, losses) = to_compact_jats(&node).await?;
    let labels = losses.iter().map(|(label, ..)| label).collect::<Vec<_>>();

    assert!(
        jats.contains(r#"<xref ref-type="fig" rid="f1">Figure 1</xref>"#),
        "internal link not encoded: {jats}"
    );
    assert!(
        labels.contains(&"Link.rel"),
        "unsupported relationship not reported: {labels:?}"
    );

    Ok(())
}

/// Section ids survive so that cross references to them still resolve
#[tokio::test]
async fn section_ids_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <sec id="s1"><p>An untitled wrapper section</p></sec>
            <sec id="s2" sec-type="methods"><title>Methods</title><p>Method</p></sec>
          </body>
          <back>
            <ack id="ack1"><p>Thanks</p></ack>
            <app-group><app id="app1"><title>Appendix</title><p>Extra</p></app></app-group>
          </back>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    for expected in [
        r#"<sec id="s1">"#,
        r#"<sec id="s2" sec-type="Methods">"#,
        r#"<ack id="ack1">"#,
        r#"<app id="app1">"#,
    ] {
        assert!(jats.contains(expected), "missing {expected}: {jats}");
    }

    Ok(())
}

/// Cross references to emitted article-part containers remain addressable
#[tokio::test]
async fn article_part_links_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>See <xref ref-type="other" rid="sa1">the response</xref></p>
          </body>
          <sub-article id="sa1">
            <front-stub><title-group><article-title>Author response</article-title></title-group></front-stub>
            <body><p>Response text</p></body>
          </sub-article>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, losses) = to_compact_jats(&node).await?;

    assert!(
        jats.contains(r#"<xref ref-type="other" rid="sa1">the response</xref>"#),
        "article-part reference not retained: {jats}"
    );
    let labels = losses.iter().map(|(label, ..)| label).collect::<Vec<_>>();
    assert!(
        !labels.contains(&"Link.target"),
        "article-part target reported as lost: {labels:?}"
    );

    Ok(())
}

/// Cross reference types reflect the element used at the target's position
#[tokio::test]
async fn section_cross_reference_types_match_emitted_elements() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <p>
              See <xref ref-type="sec" rid="ack1">the acknowledgements</xref> and
              <xref ref-type="sec" rid="app1">the nested appendix</xref>.
            </p>
            <sec id="outer">
              <title>Outer section</title>
              <sec id="app1" sec-type="appendix"><title>Nested appendix</title></sec>
            </sec>
          </body>
          <back><ack id="ack1"><p>Thanks</p></ack></back>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    for expected in [
        r#"<xref ref-type="ack" rid="ack1">the acknowledgements</xref>"#,
        r#"<xref ref-type="sec" rid="app1">the nested appendix</xref>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected}: {jats}");
    }

    Ok(())
}

/// Figure and table DOIs survive as `<object-id>` elements
#[tokio::test]
async fn object_identifiers_are_preserved() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <fig id="f1">
              <object-id pub-id-type="doi">10.1371/journal.pone.0332245.g001</object-id>
              <caption><p>A figure</p></caption>
            </fig>
            <table-wrap id="t1">
              <object-id pub-id-type="doi">10.1371/journal.pone.0332245.t001</object-id>
              <table><tbody><tr><td>Cell</td></tr></tbody></table>
            </table-wrap>
          </body>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(source, None).await?;
    let (jats, ..) = to_compact_jats(&node).await?;

    for expected in [
        r#"<object-id pub-id-type="doi">10.1371/journal.pone.0332245.g001</object-id>"#,
        r#"<object-id pub-id-type="doi">10.1371/journal.pone.0332245.t001</object-id>"#,
    ] {
        assert!(jats.contains(expected), "missing {expected}: {jats}");
    }

    Ok(())
}

/// Additional figure or table DOIs are reported when only one can be retained
#[tokio::test]
async fn duplicate_object_dois_are_reported() -> Result<()> {
    let source = r#"
        <article>
          <body>
            <fig id="f1">
              <object-id pub-id-type="doi">10.1000/first</object-id>
              <object-id pub-id-type="doi">10.1000/second</object-id>
            </fig>
          </body>
        </article>
    "#;

    let (node, info) = JatsCodec.from_str(source, None).await?;
    let labels = info
        .losses
        .iter()
        .map(|(label, ..)| label)
        .collect::<Vec<_>>();
    let (jats, ..) = to_compact_jats(&node).await?;

    assert!(
        jats.contains("10.1000/first"),
        "first DOI was not retained: {jats}"
    );
    assert!(
        !jats.contains("10.1000/second"),
        "second DOI was retained: {jats}"
    );
    assert!(
        labels.iter().any(|label| label.ends_with("/object-id")),
        "discarded DOI was not reported: {labels:?}"
    );

    Ok(())
}

/// Every populated Figure property omitted from JATS is reported as a loss
#[tokio::test]
async fn custom_figure_encoder_reports_all_losses() -> Result<()> {
    let node = art([Block::Figure(Figure {
        id_automatically: Some(true),
        label_automatically: Some(true),
        options: Box::new(FigureOptions {
            overlay_compiled: Some("<svg/>".to_string()),
            compilation_digest: Some(CompilationDigest::new(1)),
            ..Default::default()
        }),
        ..Default::default()
    })]);

    let (_, losses) = to_compact_jats(&node).await?;
    let labels = losses.iter().map(|(label, ..)| label).collect::<Vec<_>>();

    for expected in [
        "Figure.idAutomatically",
        "Figure.overlayCompiled",
        "Figure.compilationDigest",
    ] {
        assert!(labels.contains(&expected), "missing {expected}: {labels:?}");
    }
    assert!(
        !labels.contains(&"Figure.labelAutomatically"),
        "derived label state should not be a loss: {labels:?}"
    );

    Ok(())
}
