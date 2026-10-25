use std::io::Write;

use flate2::{Compression, write::GzEncoder};

use stencila_codec::{
    Losses,
    eyre::{Result, bail},
    stencila_schema::{
        Article, Author, Block, CreativeWorkType, Figure, GrantOrMonetaryGrant, Inline,
        IntegerOrString, MathBlock, Node, Note, Paragraph, Primitive, PropertyValueOrString,
        Reference, Section, SectionType, Table,
    },
};
use stencila_codec_text_trait::TextCodec;

use super::*;

/// Create a TEI document with GROBID's namespace and structure
fn tei(header: &str, body: &str, back: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<TEI xml:space="preserve" xmlns="http://www.tei-c.org/ns/1.0">
<teiHeader>{header}</teiHeader>
<text><body>{body}</body><back>{back}</back></text>
</TEI>"#
    )
}

/// Decode a TEI document to an article and its losses
fn decode_article(tei: &str) -> Result<(Article, Losses)> {
    let (node, info) = decode(tei, None)?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    Ok((article, info.losses))
}

/// Get the text of a node, without the trailing newline after blocks
fn to_text<T: TextCodec>(node: &T) -> String {
    stencila_codec_text_trait::to_text(node).trim().to_string()
}

/// Get the labels of losses
fn labels(losses: &Losses) -> Vec<&str> {
    losses.iter().map(|(label, ..)| label).collect()
}

/// Get the blocks of a type within some blocks, including within sections
fn sections(blocks: &[Block]) -> Vec<&Section> {
    blocks
        .iter()
        .filter_map(|block| match block {
            Block::Section(section) => Some(section),
            _ => None,
        })
        .collect()
}

#[test]
fn authors_with_affiliations() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
            <author role="corresp">
                <persName><forename type="first">James</forename><forename type="middle">Q</forename><surname>Boedicker</surname></persName>
                <email>boedicke@usc.edu</email>
                <idno type="ORCID">0000-0003-4107-3719</idno>
                <affiliation key="aff0">
                    <orgName type="department">Department of Physics and Astronomy</orgName>
                    <orgName type="institution">University of Southern California</orgName>
                    <address>
                        <settlement>Los Angeles</settlement>
                        <region>California</region>
                        <country key="US">United States of America</country>
                    </address>
                </affiliation>
                <affiliation key="aff1">
                    <orgName type="institution">Rice University</orgName>
                </affiliation>
            </author>
            <author><persName><forename type="first">J</forename><surname>Doe</surname></persName></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;

    let Some([Author::Person(first), Author::Person(second)]) = article.authors.as_deref() else {
        bail!("expected two people: {:?}", article.authors)
    };

    assert_eq!(first.given_names, Some(vec!["James".into(), "Q".into()]));
    assert_eq!(first.family_names, Some(vec!["Boedicker".into()]));
    assert_eq!(first.orcid.as_deref(), Some("0000-0003-4107-3719"));
    assert_eq!(first.options.emails, Some(vec!["boedicke@usc.edu".into()]));

    let Some([physics, rice]) = first.affiliations.as_deref() else {
        bail!("expected two affiliations")
    };
    assert_eq!(
        physics.name.as_deref(),
        Some("Department of Physics and Astronomy, University of Southern California")
    );
    assert_eq!(
        physics.options.address.as_ref().map(to_text).as_deref(),
        Some("Los Angeles, California, United States of America")
    );
    assert_eq!(rice.name.as_deref(), Some("Rice University"));
    assert_eq!(rice.options.address, None);

    assert_eq!(second.given_names, Some(vec!["J".into()]));
    assert_eq!(second.affiliations, None);

    // The corresponding author role, affiliation keys and country codes are
    // all represented, or say nothing more, so none is a loss
    assert!(labels(&losses).is_empty(), "{:?}", labels(&losses));

    Ok(())
}

#[test]
fn citation_resolves_to_reference() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r##"<div><head>Introduction</head><p>As shown before <ref type="bibr" target="#b0">[1]</ref> and elsewhere <ref type="bibr">[2]</ref>.</p></div>"##,
        r#"<div type="references"><listBibl>
            <biblStruct xml:id="b0">
                <analytic>
                    <title level="a" type="main">Robust control of a quorum-sensing circuit</title>
                    <author><persName><forename type="first">J</forename><forename type="middle">W</forename><surname>Williams</surname></persName></author>
                    <idno type="DOI">10.1038/msb.2008.70</idno>
                    <idno type="PMID">19096361</idno>
                </analytic>
                <monogr>
                    <title level="j">Molecular Systems Biology</title>
                    <imprint>
                        <biblScope unit="volume">4</biblScope>
                        <biblScope unit="issue">1</biblScope>
                        <biblScope unit="page" from="234" to="240" />
                        <date type="published" when="2008">2008</date>
                    </imprint>
                </monogr>
            </biblStruct>
        </listBibl></div>"#,
    ))?;

    let [Block::Section(section)] = article.content.as_slice() else {
        bail!("expected one section")
    };
    let Some(Block::Paragraph(Paragraph { content, .. })) = section.content.get(1) else {
        bail!("expected a paragraph after the heading")
    };
    let citations = content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Citation(citation) => Some(citation),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [citation] = citations.as_slice() else {
        bail!("expected one citation: {content:?}")
    };
    assert_eq!(citation.target, "b0");
    assert_eq!(
        citation.options.content.as_ref().map(to_text).as_deref(),
        Some("[1]")
    );

    // The citation that GROBID did not resolve keeps its text, and is a loss
    assert!(to_text(content).ends_with("and elsewhere [2]."));
    assert!(labels(&losses).contains(&"//TEI/text/body/div/p/ref[@type='bibr'] (unresolved)"));

    let Some([reference]) = article.references.as_deref() else {
        bail!("expected one reference")
    };
    let Reference {
        id,
        work_type,
        doi,
        title,
        date,
        is_part_of,
        options,
        ..
    } = reference;
    assert_eq!(id.as_deref(), Some("b0"));
    assert_eq!(*work_type, Some(CreativeWorkType::Article));
    assert_eq!(doi.as_deref(), Some("10.1038/msb.2008.70"));
    assert_eq!(
        title.as_ref().map(to_text).as_deref(),
        Some("Robust control of a quorum-sensing circuit")
    );
    assert_eq!(date.as_ref().map(|date| date.value.as_str()), Some("2008"));
    assert_eq!(options.page_start, Some(IntegerOrString::Integer(234)));
    assert_eq!(options.page_end, Some(IntegerOrString::Integer(240)));
    assert!(matches!(
        options.identifiers.as_deref(),
        Some([PropertyValueOrString::PropertyValue(pmid)])
            if pmid.property_id.as_deref() == Some("PMID")
    ));

    let Some(journal) = is_part_of else {
        bail!("expected the reference to be part of a journal")
    };
    assert_eq!(journal.work_type, Some(CreativeWorkType::Periodical));
    assert_eq!(
        journal.title.as_ref().map(to_text).as_deref(),
        Some("Molecular Systems Biology")
    );
    assert_eq!(
        journal.options.volume_number,
        Some(IntegerOrString::Integer(4))
    );
    assert_eq!(
        journal.options.issue_number,
        Some(IntegerOrString::Integer(1))
    );

    Ok(())
}

#[test]
fn figure_without_image_is_a_loss() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r##"<div><head>Results</head><p>See Fig <ref type="figure" target="#fig_0">1B</ref>.</p></div>
        <figure xml:id="fig_0" coords="6,200.01,368.13,371.21,6.80"><head>Fig 1 .</head><label>1</label><figDesc>Fig 1. Dynamics of quorum sensing.</figDesc><graphic coords="6,72.00,78.01,504.00,209.03" type="bitmap" /></figure>"##,
        "",
    ))?;

    let [Block::Section(section), Block::Figure(figure)] = article.content.as_slice() else {
        bail!("expected a section and a figure: {:?}", article.content)
    };

    let Figure {
        id,
        label,
        caption,
        content,
        options,
        ..
    } = figure;
    assert_eq!(id.as_deref(), Some("fig_0"));
    assert_eq!(label.as_deref(), Some("1"));
    // The label that GROBID leaves in the caption is removed
    assert_eq!(
        caption.as_ref().map(to_text).as_deref(),
        Some("Dynamics of quorum sensing.")
    );
    assert!(content.is_empty());

    // The coordinates of the figure, and of its graphic, are kept
    let coords = options
        .identifiers
        .iter()
        .flatten()
        .filter_map(|identifier| match identifier {
            PropertyValueOrString::PropertyValue(value)
                if value.property_id.as_deref() == Some("coords") =>
            {
                Some((value.options.name.clone(), value.value.clone()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        coords,
        vec![
            (
                None,
                Primitive::String("6,200.01,368.13,371.21,6.80".into())
            ),
            (
                Some("graphic".into()),
                Primitive::String("6,72.00,78.01,504.00,209.03".into())
            )
        ]
    );

    // The missing image is reported with the figure's id
    assert_eq!(
        labels(&losses),
        vec!["//TEI/text/body/figure[@xml:id='fig_0'] (no image)"]
    );

    // The reference to the figure is a link to it
    let Some(Block::Paragraph(paragraph)) = section.content.get(1) else {
        bail!("expected a paragraph")
    };
    let Some(Inline::Link(link)) = paragraph.content.get(1) else {
        bail!("expected a link: {:?}", paragraph.content)
    };
    assert_eq!(link.target, "#fig_0");
    assert_eq!(to_text(&link.content), "1B");

    Ok(())
}

#[test]
fn table_from_cells() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<figure type="table" xml:id="tab_0" coords="4,36.00,77.92,514.19,260.46"><head>Table 1 .</head><label>1</label><figDesc>Model parameters.</figDesc><table coords="4,58.79,92.78,491.40,245.61"><row><cell>Parameter</cell><cell>Value</cell></row><row><cell cols="2">k -1 = 10 min</cell></row></table><note place="foot">Values from [4].</note></figure>"#,
        "",
    ))?;

    let [Block::Table(table)] = article.content.as_slice() else {
        bail!("expected a table: {:?}", article.content)
    };
    let Table {
        id,
        label,
        caption,
        rows,
        notes,
        ..
    } = table;
    assert_eq!(id.as_deref(), Some("tab_0"));
    assert_eq!(label.as_deref(), Some("1"));
    assert_eq!(
        caption.as_ref().map(to_text).as_deref(),
        Some("Model parameters.")
    );
    assert_eq!(
        notes.as_ref().map(to_text).as_deref(),
        Some("Values from [4].")
    );

    let cells = rows
        .iter()
        .map(|row| {
            row.cells
                .iter()
                .map(|cell| to_text(&cell.content))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cells,
        vec![
            vec!["Parameter".to_string(), "Value".to_string()],
            vec!["k -1 = 10 min".to_string()]
        ]
    );
    assert_eq!(rows[1].cells[0].options.column_span, Some(2));

    // A table with cells has nothing missing, and its note is not a footnote
    // of the text
    assert!(labels(&losses).is_empty(), "{:?}", labels(&losses));

    Ok(())
}

#[test]
fn table_without_cells_is_a_loss() -> Result<()> {
    let (.., losses) = decode_article(&tei(
        "",
        r#"<figure type="table" xml:id="tab_0"><figDesc>Table 1.</figDesc><table>Name Description nic raw</table></figure>"#,
        "",
    ))?;

    assert_eq!(
        labels(&losses),
        vec![
            "//TEI/text/body/figure/table/text()",
            "//TEI/text/body/figure[@xml:id='tab_0'] (no rows)"
        ]
    );

    Ok(())
}

#[test]
fn formula_as_text_is_a_loss() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<div><head>Methods</head><formula xml:id="formula_5" coords="11,337.44,183.75,238.56,22.90">dA dt = v N(t) b I(t) <label>(13)</label></formula></div>"#,
        "",
    ))?;

    let [Block::Section(section)] = article.content.as_slice() else {
        bail!("expected a section")
    };
    let Some(Block::MathBlock(MathBlock {
        id,
        code,
        math_language,
        label,
        ..
    })) = section.content.get(1)
    else {
        bail!("expected a math block: {:?}", section.content)
    };
    assert_eq!(id.as_deref(), Some("formula_5"));
    assert_eq!(code.as_str(), "dA dt = v N(t) b I(t)");
    assert_eq!(math_language.as_deref(), Some("text"));
    assert_eq!(label.as_deref(), Some("13"));

    assert_eq!(
        labels(&losses),
        vec![
            "//TEI/text/body/div/formula (text, not TeX or MathML)",
            "//TEI/text/body/div/formula/@coords"
        ]
    );

    Ok(())
}

#[test]
fn unknown_element_is_a_loss() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<revisionDesc><change>Lost</change></revisionDesc>"#,
        r#"<div><head>Methods</head><p>Funded by <rs type="funder">NSF</rs> and <unknown>more</unknown>.</p><unknown>Kept</unknown></div>"#,
        "",
    ))?;

    // The content of an unknown element in the text is kept, without the
    // element, as when decoding JATS
    let text = to_text(&article.content);
    assert!(text.contains("Funded by NSF and more."));
    assert!(text.contains("Kept"));
    assert!(!text.contains("Lost"));

    assert_eq!(
        labels(&losses),
        vec![
            "//TEI/teiHeader/revisionDesc",
            "//TEI/text/body/div/p/rs[@type='funder']",
            "//TEI/text/body/div/p/unknown",
            "//TEI/text/body/div/unknown"
        ]
    );

    Ok(())
}

#[test]
fn sections_nested_by_numbering() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"
        <div><head n="1">Introduction</head><p>One</p></div>
        <div><head n="2">Methods</head><p>Two</p></div>
        <div><head n="2.1">Data</head><p>Two point one</p></div>
        <div><head>Unnumbered</head><p>Within two point one</p></div>
        <div><p>Continues the unnumbered section</p></div>
        <div><head n="2.2">Analysis</head></div>
        <div><head n="3">Results</head></div>
        "#,
        "",
    ))?;

    let top = sections(&article.content);
    let headings = top
        .iter()
        .map(|section| to_text(&section.content[0]))
        .collect::<Vec<_>>();
    assert_eq!(headings, vec!["Introduction", "Methods", "Results"]);
    assert_eq!(top[1].section_type, Some(SectionType::Methods));

    let methods = sections(&top[1].content);
    let Some(Block::Heading(data_heading)) = methods[0].content.first() else {
        bail!("expected a heading")
    };
    assert_eq!(data_heading.level, 2);
    assert_eq!(to_text(&methods[1].content[0]), "Analysis");

    let data = sections(&methods[0].content);
    let Some(Block::Heading(unnumbered_heading)) = data[0].content.first() else {
        bail!("expected a heading")
    };
    assert_eq!(unnumbered_heading.level, 3);
    let paragraphs = data[0].content[1..].iter().map(to_text).collect::<Vec<_>>();
    assert_eq!(
        paragraphs,
        vec!["Within two point one", "Continues the unnumbered section"]
    );

    // The numbering is used for nesting, but is not otherwise kept
    assert_eq!(labels(&losses), vec!["//TEI/text/body/div/head/@n"]);

    Ok(())
}

#[test]
fn footnote_decoded_where_referenced() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        r##"<div><head>Introduction</head><p>A claim<ref type="foot" target="#foot_0">1</ref>.<note place="foot">An inline footnote.</note></p></div>
        <note place="foot" xml:id="foot_0" n="1"><p>The footnote.</p></note>
        <note place="foot" xml:id="foot_1"><p>A running footer.</p></note>"##,
        "",
    ))?;

    let [Block::Section(section), Block::Paragraph(unreferenced)] = article.content.as_slice()
    else {
        bail!("expected a section and a paragraph: {:?}", article.content)
    };

    let Some(Block::Paragraph(paragraph)) = section.content.get(1) else {
        bail!("expected a paragraph")
    };
    let Some(Inline::Note(Note { id, content, .. })) = paragraph.content.get(1) else {
        bail!("expected a note: {:?}", paragraph.content)
    };
    assert_eq!(id.as_deref(), Some("foot_0"));
    assert_eq!(to_text(content), "The footnote.");

    // A footnote within a paragraph is decoded where it is, and only there
    let Some(Inline::Note(Note { content, .. })) = paragraph.content.get(3) else {
        bail!("expected an inline note: {:?}", paragraph.content)
    };
    assert_eq!(to_text(content), "An inline footnote.");

    // A footnote that is not referenced is at the end of the body
    let Some(Inline::Note(Note { id, .. })) = unreferenced.content.first() else {
        bail!("expected a note")
    };
    assert_eq!(id.as_deref(), Some("foot_1"));

    Ok(())
}

#[test]
fn header_metadata() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r##"<fileDesc>
            <titleStmt>
                <title level="a" type="main">Phenotypic memory in quorum sensing</title>
                <funder ref="#_grant"><orgName type="full">National Science Foundation</orgName><orgName type="abbreviated">NSF</orgName></funder>
            </titleStmt>
            <publicationStmt><publisher/><availability status="unknown"><licence target="https://creativecommons.org/licenses/by/4.0/"/></availability><date type="published" when="2024-07-08">July 8, 2024</date></publicationStmt>
            <sourceDesc><biblStruct>
                <analytic><title level="a" type="main">Phenotypic memory in quorum sensing</title></analytic>
                <monogr><title level="j" type="main">PLOS Computational Biology</title><idno type="eISSN">1553-7358</idno><imprint><biblScope unit="volume">20</biblScope><biblScope unit="issue">7</biblScope><biblScope unit="page" from="e1011696"/></imprint></monogr>
                <idno type="DOI">10.1371/journal.pcbi.1011696</idno>
                <idno type="PMID">38976752</idno>
            </biblStruct></sourceDesc>
        </fileDesc>
        <profileDesc>
            <textClass><keywords><term>quorum sensing</term><term>memory</term></keywords></textClass>
            <abstract><div><p>Quorum sensing is a regulatory mechanism.</p></div></abstract>
        </profileDesc>"##,
        "",
        r#"<listOrg type="funding"><org type="funding" xml:id="_grant"><idno type="grant-number">PHY-1753268</idno></org></listOrg>"#,
    ))?;

    assert_eq!(
        article.title.as_ref().map(to_text).as_deref(),
        Some("Phenotypic memory in quorum sensing")
    );
    assert_eq!(article.doi.as_deref(), Some("10.1371/journal.pcbi.1011696"));
    assert_eq!(
        article
            .date_published
            .as_ref()
            .map(|date| date.value.as_str()),
        Some("2024-07-08")
    );
    assert_eq!(
        article.options.keywords,
        Some(vec!["quorum sensing".into(), "memory".into()])
    );
    assert_eq!(
        article.r#abstract.as_ref().map(to_text).as_deref(),
        Some("Quorum sensing is a regulatory mechanism.")
    );
    assert_eq!(
        article
            .options
            .licenses
            .as_ref()
            .map(|licenses| licenses.len()),
        Some(1)
    );
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::String("e1011696".into()))
    );
    assert!(article.options.is_part_of.is_some());

    let Some([GrantOrMonetaryGrant::MonetaryGrant(grant)]) = article.options.funded_by.as_deref()
    else {
        bail!("expected one grant")
    };
    assert_eq!(grant.options.funders.as_ref().map(Vec::len), Some(1));
    assert_eq!(article.options.funders.as_ref().map(Vec::len), Some(1));

    // The title repeated in the source description is not a loss
    assert!(labels(&losses).is_empty(), "{:?}", labels(&losses));

    Ok(())
}

#[test]
fn back_matter_sections() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        "",
        r#"
        <div type="acknowledgement"><div><head>Acknowledgements</head><p>We thank everyone.</p></div></div>
        <div type="availability"><div><p>All data are available.</p></div></div>
        <div type="annex"><div><head>Appendix A</head><p>More detail.</p></div></div>
        "#,
    ))?;

    let types = sections(&article.content)
        .iter()
        .map(|section| section.section_type)
        .collect::<Vec<_>>();
    assert_eq!(
        types,
        vec![
            Some(SectionType::Acknowledgements),
            Some(SectionType::DataAvailability),
            Some(SectionType::Appendix)
        ]
    );

    // The acknowledgements are a heading and paragraph, as decoded from JATS
    let [Block::Section(acknowledgements), ..] = article.content.as_slice() else {
        bail!("expected sections")
    };
    assert!(matches!(
        acknowledgements.content.as_slice(),
        [Block::Heading(..), Block::Paragraph(..)]
    ));

    Ok(())
}

#[test]
fn chapter_in_book() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        "",
        r#"<div type="references"><listBibl><biblStruct xml:id="b1">
            <analytic><title level="a" type="main">A chapter</title></analytic>
            <monogr>
                <title level="m">A book</title>
                <editor><persName><forename>A</forename><surname>Editor</surname></persName></editor>
                <imprint><publisher>Springer</publisher><pubPlace>Berlin</pubPlace><date when="2001"/><biblScope unit="page">10-20</biblScope></imprint>
            </monogr>
        </biblStruct></listBibl></div>"#,
    ))?;

    let Some([reference]) = article.references.as_deref() else {
        bail!("expected one reference")
    };
    assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
    assert_eq!(
        reference.options.page_start,
        Some(IntegerOrString::Integer(10))
    );
    assert_eq!(
        reference.options.page_end,
        Some(IntegerOrString::Integer(20))
    );

    let Some(book) = &reference.is_part_of else {
        bail!("expected the reference to be part of a book")
    };
    assert_eq!(book.work_type, Some(CreativeWorkType::Book));
    assert_eq!(book.options.editors.as_ref().map(Vec::len), Some(1));
    assert!(book.options.publisher.is_some());

    Ok(())
}

#[tokio::test]
async fn gzip_compressed() -> Result<()> {
    let tei = tei(
        r#"<fileDesc><titleStmt><title level="a" type="main">Compressed</title></titleStmt></fileDesc>"#,
        "",
        "",
    );

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(tei.as_bytes())?;
    let bytes = encoder.finish()?;

    // Content fetched from a URL is only decoded as bytes, so that compressed
    // content is decompressed, if the codec does not support strings
    assert!(!TeiCodec.supports_from_string());

    let (node, ..) = TeiCodec.from_bytes(&bytes, None).await?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    assert_eq!(
        article.title.as_ref().map(to_text).as_deref(),
        Some("Compressed")
    );

    Ok(())
}

#[test]
fn not_tei() {
    assert!(decode("<article/>", None).is_err());
}

#[test]
fn footnote_referenced_before_it_is_decoded_once() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        r##"<p>See<ref type="foot" target="#foot_0">1</ref> and <note place="foot" xml:id="foot_0">The note.</note></p>"##,
        "",
    ))?;

    let [Block::Paragraph(Paragraph { content, .. })] = article.content.as_slice() else {
        bail!("expected a paragraph: {:?}", article.content)
    };
    let notes = content
        .iter()
        .filter(|inline| matches!(inline, Inline::Note(..)))
        .count();
    assert_eq!(notes, 1, "{content:?}");

    Ok(())
}

#[test]
fn abstract_heading_within_division_is_dropped() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        r#"<profileDesc><abstract><div><head>Abstract</head><p>Hello</p></div></abstract></profileDesc>"#,
        "",
        "",
    ))?;

    let Some([Block::Paragraph(paragraph)]) = article.r#abstract.as_deref() else {
        bail!("expected a paragraph: {:?}", article.r#abstract)
    };
    assert_eq!(to_text(paragraph), "Hello");

    Ok(())
}

#[test]
fn section_type_from_heading_if_not_from_type() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<div type="weird"><head>Methods</head><p>Text</p></div>"#,
        "",
    ))?;

    let [section] = sections(&article.content)[..] else {
        bail!("expected a section")
    };
    assert_eq!(section.section_type, Some(SectionType::Methods));
    assert!(labels(&losses).contains(&"//TEI/text/body/div/@type"));

    Ok(())
}

#[test]
fn pages_of_article() -> Result<()> {
    let header = |scopes: &str| {
        tei(
            &format!(
                r#"<fileDesc><sourceDesc><biblStruct><monogr><imprint>{scopes}</imprint></monogr></biblStruct></sourceDesc></fileDesc>"#
            ),
            "",
            "",
        )
    };

    // A leading zero is kept
    let (article, ..) = decode_article(&header(r#"<biblScope unit="page">056708</biblScope>"#))?;
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::String("056708".into()))
    );

    // A range in the text is split
    let (article, ..) = decode_article(&header(r#"<biblScope unit="page">12-20</biblScope>"#))?;
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::Integer(12))
    );
    assert_eq!(article.options.page_end, Some(IntegerOrString::Integer(20)));

    // A later scope with only a first page does not remove the last page
    let (article, ..) = decode_article(&header(
        r#"<biblScope unit="page" from="1" to="9"/><biblScope unit="page" from="1"/><biblScope unit="volume">07</biblScope>"#,
    ))?;
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::Integer(1))
    );
    assert_eq!(article.options.page_end, Some(IntegerOrString::Integer(9)));

    Ok(())
}

#[test]
fn pages_of_reference() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        "",
        r#"<listBibl><biblStruct xml:id="b0"><monogr><title level="m">A book</title><imprint>
            <biblScope unit="page" from="1" to="9"/>
            <biblScope unit="page" from="1"/>
        </imprint></monogr></biblStruct></listBibl>"#,
    ))?;

    let Some([Reference { options, .. }]) = article.references.as_deref() else {
        bail!("expected a reference")
    };
    assert_eq!(options.page_start, Some(IntegerOrString::Integer(1)));
    assert_eq!(options.page_end, Some(IntegerOrString::Integer(9)));

    Ok(())
}

#[test]
fn ror_with_http_prefix() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic><author>
            <persName><surname>Doe</surname></persName>
            <affiliation><orgName>Uni</orgName><idno type="ROR">http://ror.org/02mhbdp94</idno></affiliation>
        </author></analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;

    let Some([Author::Person(person)]) = article.authors.as_deref() else {
        bail!("expected a person")
    };
    let Some([affiliation]) = person.affiliations.as_deref() else {
        bail!("expected an affiliation")
    };
    assert_eq!(affiliation.ror.as_deref(), Some("02mhbdp94"));

    Ok(())
}

#[test]
fn figure_label_from_head() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<figure xml:id="fig_0"><head>Figure 3</head><figDesc>Figure 3. Results of the experiment.</figDesc></figure>
        <figure xml:id="fig_1"><head>Figure legends</head><figDesc>Some legends.</figDesc></figure>"#,
        "",
    ))?;

    let [Block::Figure(labelled), Block::Figure(unlabelled)] = article.content.as_slice() else {
        bail!("expected two figures: {:?}", article.content)
    };

    assert_eq!(labelled.label.as_deref(), Some("3"));
    assert_eq!(labelled.label_automatically, Some(false));
    assert_eq!(
        labelled.caption.as_ref().map(to_text).as_deref(),
        Some("Results of the experiment.")
    );

    // A heading that gives no label is lost
    assert_eq!(unlabelled.label, None);
    assert!(labels(&losses).contains(&"//TEI/text/body/figure/head"));

    Ok(())
}

#[test]
fn citation_of_several_references() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        r##"<p>As shown <ref type="bibr" target="#b0 #b1">[1,2]</ref>.</p>"##,
        r#"<listBibl>
            <biblStruct xml:id="b0"><monogr><title level="m">One</title></monogr></biblStruct>
            <biblStruct xml:id="b1"><monogr><title level="m">Two</title></monogr></biblStruct>
        </listBibl>"#,
    ))?;

    let [Block::Paragraph(Paragraph { content, .. })] = article.content.as_slice() else {
        bail!("expected a paragraph: {:?}", article.content)
    };
    let Some(Inline::CitationGroup(group)) = content.get(1) else {
        bail!("expected a citation group: {content:?}")
    };
    let targets = group
        .items
        .iter()
        .map(|citation| citation.target.as_str())
        .collect::<Vec<_>>();
    assert_eq!(targets, vec!["b0", "b1"]);
    assert_eq!(
        group.content.as_ref().map(to_text).as_deref(),
        Some("[1,2]")
    );

    Ok(())
}

#[test]
fn formula_markup_text_is_not_duplicated() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        r#"<formula><hi rend="italic">x</hi> + 1</formula>"#,
        "",
    ))?;

    let [Block::MathBlock(MathBlock { code, .. })] = article.content.as_slice() else {
        bail!("expected a math block: {:?}", article.content)
    };
    assert_eq!(code.as_str(), "x + 1");

    Ok(())
}

#[test]
fn divisions_of_text_in_order() -> Result<()> {
    let (article, ..) = decode_article(
        r#"<TEI xmlns="http://www.tei-c.org/ns/1.0"><text>
            <div><head>Preface</head></div>
            <body><div><head>Body</head></div></body>
            <div><head>Appendix</head></div>
            <back><div type="acknowledgement"><head>Acknowledgements</head><p>Thanks</p></div></back>
            <div><head>Afterword</head></div>
        </text></TEI>"#,
    )?;

    let headings = article
        .content
        .iter()
        .map(|block| match block {
            Block::Section(section) => section.content.first().map(to_text).unwrap_or_default(),
            block => to_text(block),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        headings,
        vec![
            "Preface",
            "Body",
            "Appendix",
            "Acknowledgements",
            "Afterword"
        ]
    );

    Ok(())
}

#[test]
fn corpus_of_one_or_more() -> Result<()> {
    let one = r#"<teiCorpus xmlns="http://www.tei-c.org/ns/1.0"><TEI><text><body><p>One</p></body></text></TEI></teiCorpus>"#;
    let (article, ..) = decode_article(one)?;
    assert_eq!(to_text(&article.content), "One");

    let two = r#"<teiCorpus xmlns="http://www.tei-c.org/ns/1.0"><TEI/><TEI/></teiCorpus>"#;
    assert!(decode(two, None).is_err());

    Ok(())
}

#[test]
fn formula_text_has_renderable_mathml() -> Result<()> {
    let (article, ..) = decode_article(&tei(
        "",
        r#"<formula xml:id="eq1">x &lt; y &amp; y &gt; z<label>(1)</label></formula>
        <p>Inline <formula>α &lt; β</formula>.</p>"#,
        "",
    ))?;

    let [Block::MathBlock(block), Block::Paragraph(paragraph)] = article.content.as_slice() else {
        bail!("expected display math and a paragraph")
    };
    let Some(Inline::MathInline(inline)) = paragraph.content.get(1) else {
        bail!("expected inline math")
    };

    let dom = stencila_codec_dom_trait::to_dom(&article);
    assert_eq!(dom.matches("slot=mathml").count(), 2);
    for (code, mathml, expected) in [
        (&block.code, &block.options.mathml, "x < y & y > z"),
        (&inline.code, &inline.options.mathml, "α < β"),
    ] {
        assert_eq!(code.as_str(), expected);
        let Some(mathml) = mathml else {
            bail!("expected a renderable fallback for {expected}")
        };
        let xml = roxmltree::Document::parse(mathml)?;
        assert!(
            xml.root_element()
                .has_tag_name(("http://www.w3.org/1998/Math/MathML", "math"))
        );
        let Some(text) = xml.descendants().find(|node| node.has_tag_name("mtext")) else {
            bail!("expected the extracted formula as MathML text")
        };
        assert_eq!(text.text(), Some(expected));
        assert_eq!(
            xml.descendants().filter(|node| node.is_element()).count(),
            2
        );
        assert!(dom.contains(mathml));
    }

    // The same fallback must survive export to another document format.
    let (jats, ..) = stencila_codec_jats::encode(&Node::Article(article), None)?;
    let xml = roxmltree::Document::parse(&jats)?;
    let formulas = xml
        .descendants()
        .filter(|node| node.has_tag_name("mtext"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(formulas, ["x < y & y > z", "α < β"]);

    Ok(())
}

#[test]
fn sentence_boundaries_are_preserved() -> Result<()> {
    for (content, expected) in [
        (
            "<s>First sentence.</s><s>Second sentence.</s>",
            "First sentence. Second sentence.",
        ),
        (
            "<s> First sentence. </s>\n  <s> Second sentence. </s>",
            "First sentence. Second sentence.",
        ),
        (
            r#"<s><hi rend="italic">First</hi> sentence.</s><s><ref target="https://example.org">Second</ref> sentence.</s>"#,
            "First sentence. Second sentence.",
        ),
        (
            r#"<s>A <hi rend="bold">co</hi>efficient.</s><s>Done.</s>"#,
            "A coefficient. Done.",
        ),
    ] {
        let (article, losses) = decode_article(&tei("", &format!("<p>{content}</p>"), ""))?;
        assert_eq!(to_text(&article.content), expected);
        assert!(losses.is_empty(), "{:?}", labels(&losses));
    }

    Ok(())
}
