use std::io::Write;

use flate2::{Compression, write::GzEncoder};

use stencila_codec::{
    Losses, PageMap,
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Article, Author, Block, CreativeWorkType, CreativeWorkVariant, Figure,
        GrantOrMonetaryGrant, Inline, InspectNode, IntegerOrString, MathBlock, Node, Note,
        Paragraph, PropertyValueOrString, Reference, Section, SectionType, Table,
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

/// Decode a TEI document to an article and its losses and page map
///
/// The page map is of the nodes of this article only: decoding again gives
/// nodes with other ids.
fn decode_article_with_page_map(tei: &str) -> Result<(Article, Losses, PageMap)> {
    let (node, info) = decode(tei, None)?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    Ok((article, info.losses, info.page_map))
}

/// Get the boxes of a node in a page map, and the part of the node that each
/// is of, with each box as `page,x,y,width,height`
fn boxes(page_map: &PageMap, node: &dyn InspectNode) -> Result<Vec<(Option<String>, String)>> {
    let node_id = node.node_id().ok_or_eyre("expected a node with an id")?;
    Ok(page_map
        .entries_for(&node_id)
        .map(|entry| {
            let boxes = entry
                .boxes
                .iter()
                .map(|page_box| {
                    format!(
                        "{},{},{},{},{}",
                        page_box.page, page_box.x, page_box.y, page_box.width, page_box.height
                    )
                })
                .collect::<Vec<_>>()
                .join(";");
            (entry.part.clone(), boxes)
        })
        .collect())
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
fn dummy_author_reports_orphan_affiliations() -> Result<()> {
    // Reproduce the brief's nine author elements: eight real authors and a
    // dummy author with two orphan affiliations, without relying on the full
    // ScienceBeam document.
    let authors = (1..=8)
        .map(|index| {
            format!("<author><persName><surname>Author {index}</surname></persName></author>")
        })
        .collect::<String>();
    let (article, losses) = decode_article(&tei(
        &format!(
            r#"<fileDesc><sourceDesc><biblStruct><analytic>
                {authors}
                <author>
                    <note type="dummy_author">Dummy author for orphan affiliations</note>
                    <affiliation><orgName type="institution">University of Auckland</orgName></affiliation>
                    <affiliation><orgName type="institution">University of Tasmania</orgName></affiliation>
                </author>
            </analytic></biblStruct></sourceDesc></fileDesc>"#,
        ),
        "",
        "",
    ))?;

    let authors = article.authors.ok_or_eyre("expected authors")?;
    assert_eq!(authors.len(), 8);
    for (index, author) in authors.iter().enumerate() {
        let Author::Person(person) = author else {
            bail!("expected a person")
        };
        assert_eq!(
            person.family_names,
            Some(vec![format!("Author {}", index + 1)])
        );
        assert_eq!(person.affiliations, None);
    }
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        vec![
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation: University of Auckland)",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation: University of Tasmania)",
                1
            ),
        ]
    );

    Ok(())
}

#[test]
fn dummy_author_reports_raw_and_empty_affiliations() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
            <author>
                <affiliation xml:id="aff0" coords="1,1,2,3,4">
                    <note type="raw_affiliation">University of Auckland,
                        New Zealand</note>
                </affiliation>
                <affiliation> </affiliation>
                <affiliation/>
                <note type="dummy_author"/>
                <note type="other">Other information</note>
            </author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;

    assert!(article.authors.is_none());
    assert!(page_map.entries().is_empty());
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        vec![
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation)",
                2
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation: University of Auckland, New Zealand)",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation/@coords",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation/@id",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/note",
                1
            ),
        ]
    );

    Ok(())
}

#[test]
fn dummy_author_names_orphan_affiliations_as_decoded() -> Result<()> {
    // Structured as by ScienceBeam and GROBID, and compact, so that the text of
    // the parts would run together if not joined as when decoded for a person
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
            <author><note type="dummy_author">Dummy author for orphan affiliations</note><affiliation key="aff1"><note type="raw_affiliation"><label>1</label> Institute for Marine and Antarctic Studies, University of Tasmania, Hobart, Australia</note><orgName type="department" coords="1,10,20,30,5">Institute for Marine and Antarctic Studies</orgName><orgName type="institution">University of Tasmania</orgName><address><settlement>Hobart</settlement><country key="AU">Australia</country></address></affiliation><affiliation><address><settlement>Auckland</settlement><country>New Zealand</country></address></affiliation></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;

    assert!(article.authors.is_none());
    assert!(page_map.entries().is_empty());
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        vec![
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation: Auckland, New Zealand)",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation (orphan affiliation: Institute for Marine and Antarctic Studies, University of Tasmania)",
                1
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/affiliation/orgName/@coords",
                1
            ),
        ]
    );

    Ok(())
}

#[test]
fn dummy_author_is_recognized_by_its_note_type() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
            <author>
                <persName><surname>Doe</surname></persName>
                <note type="other">Dummy author for orphan affiliations</note>
            </author>
            <author>
                <orgName>Consortium</orgName>
                <note type="dummy_author_extra"/>
            </author>
            <author>
                <persName>Placeholder</persName>
                <note type="dummy_author"/>
            </author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;

    let Some([Author::Person(person), Author::Organization(organization)]) =
        article.authors.as_deref()
    else {
        bail!(
            "expected a person and an organization: {:?}",
            article.authors
        )
    };
    assert_eq!(person.family_names, Some(vec!["Doe".into()]));
    assert_eq!(organization.name.as_deref(), Some("Consortium"));
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        vec![
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/note",
                2
            ),
            (
                "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/persName",
                1
            ),
        ]
    );

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
    let xml = tei(
        "",
        r##"<div><head>Results</head><p>See Fig <ref type="figure" target="#fig_0">1B</ref>.</p></div>
        <figure xml:id="fig_0" coords="6,200.01,368.13,371.21,6.80"><head>Fig 1 .</head><label>1</label><figDesc>Fig 1. Dynamics of quorum sensing.</figDesc><graphic coords="6,72.00,78.01,504.00,209.03" type="bitmap" /></figure>"##,
        "",
    );
    let (article, losses, page_map) = decode_article_with_page_map(&xml)?;

    let [Block::Section(section), Block::Figure(figure)] = article.content.as_slice() else {
        bail!("expected a section and a figure: {:?}", article.content)
    };

    let Figure {
        id,
        label,
        caption,
        content,
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

    // The coordinates of the figure, and of its graphic, are in the page map
    assert_eq!(
        boxes(&page_map, figure)?,
        vec![
            (None, "6,200.01,368.13,371.21,6.8".into()),
            (Some("graphic".into()), "6,72,78.01,504,209.03".into())
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
fn figure_notes_follow_images() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<figure>
            <figDesc>Results.</figDesc>
            <note place="foot">First <hi rend="italic">note</hi>.</note>
            <graphic url="first.png"/>
            <note><p>Second note.</p><list><item>Detail.</item></list></note>
            <graphic url="second.png"/>
        </figure>"#,
        "",
    ))?;

    let [Block::Figure(figure)] = article.content.as_slice() else {
        bail!("expected only a figure: {:?}", article.content)
    };
    let [
        Block::ImageObject(first_image),
        Block::ImageObject(second_image),
        Block::Paragraph(first_note),
        Block::Paragraph(second_note),
        Block::List(list),
    ] = figure.content.as_slice()
    else {
        bail!(
            "expected images followed by note blocks: {:?}",
            figure.content
        )
    };
    assert_eq!(first_image.content_url, "first.png");
    assert_eq!(second_image.content_url, "second.png");
    assert_eq!(to_text(first_note), "First note.");
    assert!(matches!(
        first_note.content.get(1),
        Some(Inline::Emphasis(_))
    ));
    assert_eq!(to_text(second_note), "Second note.");
    assert_eq!(
        list.items.iter().map(to_text).collect::<Vec<_>>(),
        ["Detail."]
    );
    assert_eq!(
        figure.caption.as_ref().map(to_text).as_deref(),
        Some("Results.")
    );
    assert!(losses.is_empty(), "{:?}", labels(&losses));

    Ok(())
}

#[test]
fn figure_notes_without_image_preserve_text_and_report_missing_image() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<figure xml:id="fig_0">
            <graphic type="bitmap"/>
            <note type="&lt;content&gt;">Induction and Antiviral Response 8,84E00 5,93E-02 DDX58, IFIH1</note>
            <note place="foot">Values from the results table.</note>
        </figure>"#,
        "",
    ))?;

    let [Block::Figure(figure)] = article.content.as_slice() else {
        bail!("expected only a figure: {:?}", article.content)
    };
    let [Block::Paragraph(content), Block::Paragraph(note)] = figure.content.as_slice() else {
        bail!("expected two paragraphs: {:?}", figure.content)
    };
    assert_eq!(
        to_text(content),
        "Induction and Antiviral Response 8,84E00 5,93E-02 DDX58, IFIH1"
    );
    assert_eq!(to_text(note), "Values from the results table.");
    assert_eq!(
        labels(&losses),
        vec![
            "//TEI/text/body/figure/note/@type",
            "//TEI/text/body/figure[@xml:id='fig_0'] (no image)"
        ]
    );

    Ok(())
}

#[test]
fn figure_note_ids_and_blocks_are_preserved() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        "",
        r##"<figure xml:id="fig_0">
            <figDesc>See <ref type="foot" target="#fn1">a</ref> and <ref type="foot" target="#fn2">b</ref>.</figDesc>
            <graphic url="figure.png"/>
            <note xml:id="fn1" place="foot" coords="1,10,20,30,40"><p>First note.</p><p>Continued.</p></note>
            <note id="fn2" place="foot">Second note.</note>
        </figure>"##,
        "",
    ))?;

    let [Block::Figure(figure)] = article.content.as_slice() else {
        bail!("expected only a figure: {:?}", article.content)
    };
    let [
        Block::ImageObject(_),
        Block::Paragraph(first),
        Block::Paragraph(second),
    ] = figure.content.as_slice()
    else {
        bail!(
            "expected an image followed by two notes: {:?}",
            figure.content
        )
    };
    let ([Inline::Note(first)], [Inline::Note(second)]) =
        (first.content.as_slice(), second.content.as_slice())
    else {
        bail!("expected identified figure notes")
    };
    assert_eq!(first.id.as_deref(), Some("fn1"));
    assert_eq!(second.id.as_deref(), Some("fn2"));
    assert_eq!(
        first.content.iter().map(to_text).collect::<Vec<_>>(),
        ["First note.", "Continued."]
    );
    assert_eq!(to_text(&second.content), "Second note.");
    assert_eq!(
        boxes(&page_map, first)?,
        vec![(None, "1,10,20,30,40".into())]
    );
    assert!(losses.is_empty(), "{:?}", labels(&losses));
    assert_jats_link_targets(article, &["fn1", "fn2"])
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
        vec!["//TEI/text/body/div/formula (text, not TeX or MathML)"]
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
fn reference_urls() -> Result<()> {
    let url = "https://grigoriefflab.umassmed.edu/diffmap";
    for location in ["", "analytic", "monogr"] {
        let parent = if location.is_empty() {
            "//TEI/text/back/listBibl/biblStruct".to_string()
        } else {
            format!("//TEI/text/back/listBibl/biblStruct/{location}")
        };
        for (content, lost) in [
            (format!(r#"<ref type="url"> {url} </ref>"#), &[][..]),
            (
                format!(r#"<ref type="url" target="{url}">{url}</ref>"#),
                &[],
            ),
            (
                format!(r#"<ref type="url" target=" {url} ">Website</ref>"#),
                &["ref/text()"],
            ),
            (format!(r#"<ref type="url" target=" ">{url}</ref>"#), &[]),
            (
                format!(r#"<ref type="url" xml:id="u1" coords="1,2,3,4,5">{url}</ref>"#),
                &["ref/@coords", "ref/@id"],
            ),
            (
                format!(r#"<ptr target="{url}"/><ref type="url">https://example.org</ref>"#),
                &["ref"],
            ),
            (
                format!(r#"<ref type="url">{url}</ref><ptr target="https://example.org"/>"#),
                &["ptr"],
            ),
            (
                format!(r#"<ref type="url">{url}</ref><ptr target="{url}"/>"#),
                &[],
            ),
        ] {
            let content = if location.is_empty() {
                content
            } else {
                format!("<{location}>{content}</{location}>")
            };
            let (article, losses) = decode_article(&tei(
                "",
                "",
                &format!("<listBibl><biblStruct>{content}</biblStruct></listBibl>"),
            ))?;
            let Some([reference]) = article.references.as_deref() else {
                bail!("expected one reference")
            };
            assert_eq!(reference.url.as_deref(), Some(url), "{content}");
            assert_eq!(
                losses
                    .iter()
                    .map(|(label, count)| (label.to_string(), count))
                    .collect::<Vec<_>>(),
                lost.iter()
                    .map(|lost| (format!("{parent}/{lost}"), 1))
                    .collect::<Vec<_>>(),
                "{content}"
            );
        }
    }

    // Empty URLs and internal pointers must not prevent a later URL being kept.
    let (article, losses) = decode_article(&tei(
        "",
        "",
        &format!(
            r##"<listBibl><biblStruct>
                <ref type="url"> </ref>
                <ref type="url" target="#b0">Internal link</ref>
                <ref type="url">#b0</ref>
                <ref type="bibr" target="#b0">A citation</ref>
                <ptr target="#b0"/>
                <ref type="url">{url}</ref>
            </biblStruct></listBibl>"##
        ),
    ))?;
    let Some([reference]) = article.references.as_deref() else {
        bail!("expected one reference")
    };
    assert_eq!(reference.url.as_deref(), Some(url));
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        vec![
            ("//TEI/text/back/listBibl/biblStruct/ptr", 1),
            ("//TEI/text/back/listBibl/biblStruct/ref", 3),
        ]
    );

    Ok(())
}

#[test]
fn reference_published_date() -> Result<()> {
    for (dates, expected) in [
        (
            r#"<date type="published" when="2015"/><date when="2023"/>"#,
            "2015",
        ),
        (
            r#"<date when="2023"/><date type="published" when="2015"/>"#,
            "2015",
        ),
        (
            r#"<date type="published">2015</date><date type="accessed" when="2023"/>"#,
            "2015",
        ),
        (r#"<date when="2015"/><date when="2023"/>"#, "2023"),
        (
            r#"<date when="2023"/><date type="published" when=" "/>"#,
            "2023",
        ),
        (
            r#"<date type="published" when="2015"/><date type="published" when=" "/>"#,
            "2015",
        ),
    ] {
        let (article, losses) = decode_article(&tei(
            "",
            "",
            &format!(
                "<listBibl><biblStruct><monogr><imprint>{dates}</imprint></monogr></biblStruct></listBibl>"
            ),
        ))?;
        let Some([reference]) = article.references.as_deref() else {
            bail!("expected one reference")
        };
        assert_eq!(
            reference.date.as_ref().map(|date| date.value.as_str()),
            Some(expected),
            "{dates}"
        );
        assert!(labels(&losses).is_empty(), "{:?}", labels(&losses));
    }

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

/// Get the language, and the text of each block of the abstract, of each part
/// of an article
fn abstract_parts(article: &Article) -> Result<Vec<(Option<String>, Vec<String>)>> {
    article
        .options
        .parts
        .iter()
        .flatten()
        .map(|part| {
            let CreativeWorkVariant::Article(part) = part else {
                bail!("expected an article part")
            };
            let language = serde_json::to_value(part)?["inLanguage"]
                .as_str()
                .map(String::from);
            let content = part.r#abstract.iter().flatten().map(to_text).collect();
            Ok((language, content))
        })
        .collect()
}

#[test]
fn consecutive_abstracts_in_same_language_are_joined() -> Result<()> {
    // The language of the abstract is lost, but only on the abstracts where
    // they do not inherit it (as from GROBID's `teiHeader`)
    for (profile_language, language, expected_losses) in [
        ("", "", vec![]),
        (
            r#" xml:lang="en""#,
            r#" xml:lang="en""#,
            vec![("//TEI/teiHeader/profileDesc/@lang", 1)],
        ),
        (
            "",
            r#" xml:lang="en""#,
            vec![("//TEI/teiHeader/profileDesc/abstract/@lang", 2)],
        ),
    ] {
        let (article, losses) = decode_article(&tei(
            &format!(
                r#"<profileDesc{profile_language}>
                    <abstract{language}><p>The database provides</p></abstract>
                    <abstract{language}><p>a systematic view of neurodevelopmental disorders.</p></abstract>
                </profileDesc>"#
            ),
            "",
            "",
        ))?;

        let Some([Block::Paragraph(first), Block::Paragraph(second)]) =
            article.r#abstract.as_deref()
        else {
            bail!(
                "expected both abstract paragraphs: {:?}",
                article.r#abstract
            )
        };
        assert_eq!(to_text(first), "The database provides");
        assert_eq!(
            to_text(second),
            "a systematic view of neurodevelopmental disorders."
        );
        assert!(article.options.parts.is_none());
        assert_eq!(losses.iter().collect::<Vec<_>>(), expected_losses);
    }

    Ok(())
}

#[test]
fn abstract_language_groups_become_parts_with_coordinates() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<profileDesc>
            <abstract xml:lang="en"><p coords="1,1,2,3,4">First</p></abstract>
            <!-- A comment does not interrupt a continuation. -->
            <abstract xml:lang="EN"><div><head>Abstract</head><p coords="1,2,2,3,4">Second</p></div></abstract>
            <abstract xml:lang="fr"><p coords="1,3,2,3,4">Troisième</p></abstract>
            <abstract xml:lang="fr"><div><head>Abstract</head><p coords="1,4,2,3,4">Quatrième</p></div></abstract>
            <abstract xml:lang="en"><p coords="1,5,2,3,4">Fifth</p></abstract>
            <abstract><p coords="1,6,2,3,4">Sixth</p></abstract>
        </profileDesc>"#,
        "",
        "",
    ))?;

    let primary = article
        .r#abstract
        .as_ref()
        .ok_or_eyre("expected an abstract")?;
    assert_eq!(
        primary.iter().map(to_text).collect::<Vec<_>>(),
        ["First", "Second"]
    );

    let expected: [(Option<String>, Vec<String>); 3] = [
        (
            Some("fr".into()),
            vec!["Troisième".into(), "Quatrième".into()],
        ),
        (Some("en".into()), vec!["Fifth".into()]),
        (None, vec!["Sixth".into()]),
    ];
    assert_eq!(abstract_parts(&article)?, expected);

    let parts = article
        .options
        .parts
        .as_ref()
        .ok_or_eyre("expected parts")?;
    let mut paragraphs = primary.iter().collect::<Vec<_>>();
    for part in parts {
        let CreativeWorkVariant::Article(part) = part else {
            bail!("expected an article part")
        };
        assert!(part.content.is_empty());
        paragraphs.extend(part.r#abstract.iter().flatten());
    }
    for (index, paragraph) in paragraphs.into_iter().enumerate() {
        assert!(matches!(paragraph, Block::Paragraph(..)));
        assert_eq!(
            boxes(&page_map, paragraph)?,
            vec![(None, format!("1,{},2,3,4", index + 1))]
        );
    }
    assert_eq!(page_map.entries().len(), 6);
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        [("//TEI/teiHeader/profileDesc/abstract/@lang", 2)]
    );

    Ok(())
}

#[test]
fn abstract_languages_are_inherited_and_can_be_reset() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<profileDesc xml:lang="en">
            <abstract><p>Inherited English</p></abstract>
            <abstract xml:lang="en"><p>Explicit English</p></abstract>
            <abstract xml:lang="fr"><p>French</p></abstract>
            <abstract><p>English again</p></abstract>
            <abstract xml:lang=""><p>Unspecified language</p></abstract>
        </profileDesc>"#,
        "",
        "",
    ))?;

    let primary = article
        .r#abstract
        .as_ref()
        .ok_or_eyre("expected an abstract")?;
    assert_eq!(
        primary.iter().map(to_text).collect::<Vec<_>>(),
        ["Inherited English", "Explicit English"]
    );
    let expected: [(Option<String>, Vec<String>); 3] = [
        (Some("fr".into()), vec!["French".into()]),
        (Some("en".into()), vec!["English again".into()]),
        (None, vec!["Unspecified language".into()]),
    ];
    assert_eq!(abstract_parts(&article)?, expected);
    // The language of the profile as a whole, which is also that of the main
    // abstract, is unsupported, but the parts are decoded using it
    assert_eq!(labels(&losses), ["//TEI/teiHeader/profileDesc/@lang"]);

    Ok(())
}

#[test]
fn main_abstract_language_is_lost_where_not_inherited() -> Result<()> {
    for language in ["de", ""] {
        let (article, losses) = decode_article(&tei(
            &format!(
                r#"<profileDesc xml:lang="en">
                    <abstract xml:lang="{language}"><p>Main</p></abstract>
                    <abstract><p>English</p></abstract>
                </profileDesc>"#
            ),
            "",
            "",
        ))?;

        assert_eq!(
            article.r#abstract.as_ref().map(to_text).as_deref(),
            Some("Main")
        );
        let expected: [(Option<String>, Vec<String>); 1] =
            [(Some("en".into()), vec!["English".into()])];
        assert_eq!(abstract_parts(&article)?, expected);
        assert_eq!(
            losses.iter().collect::<Vec<_>>(),
            [
                ("//TEI/teiHeader/profileDesc/@lang", 1),
                ("//TEI/teiHeader/profileDesc/abstract/@lang", 1)
            ]
        );
    }

    Ok(())
}

#[test]
fn abstract_attributes_other_than_xml_lang_are_lost() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<profileDesc xml:lang="en" xmlns:custom="https://example.org/custom">
            <abstract xml:id="abs" type="main" coords="1,1,2,3,4" xml:lang="en" lang="fr" custom:lang="de"><p>Content</p></abstract>
        </profileDesc>"#,
        "",
        "",
    ))?;

    assert_eq!(
        article.r#abstract.as_ref().map(to_text).as_deref(),
        Some("Content")
    );
    assert!(article.options.parts.is_none());
    // The `xml:lang` is the same as the inherited language, so the two
    // `lang` losses are of the other attributes with that local name
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        [
            ("//TEI/teiHeader/profileDesc/@lang", 1),
            ("//TEI/teiHeader/profileDesc/abstract/@coords", 1),
            ("//TEI/teiHeader/profileDesc/abstract/@id", 1),
            ("//TEI/teiHeader/profileDesc/abstract/@lang", 2),
            ("//TEI/teiHeader/profileDesc/abstract/@type", 1),
        ]
    );

    Ok(())
}

#[test]
fn empty_abstract_groups_do_not_create_parts() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<profileDesc>
            <abstract xml:lang="fr"/>
            <abstract xml:lang="en"><p>First</p></abstract>
            <abstract xml:lang="en"/>
            <abstract xml:lang="en"><p>Second</p></abstract>
            <abstract xml:lang="fr"><head>Abstract</head></abstract>
        </profileDesc>"#,
        "",
        "",
    ))?;

    let primary = article
        .r#abstract
        .as_ref()
        .ok_or_eyre("expected an abstract")?;
    assert_eq!(
        primary.iter().map(to_text).collect::<Vec<_>>(),
        ["First", "Second"]
    );
    assert!(article.options.parts.is_none());
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        [("//TEI/teiHeader/profileDesc/abstract/@lang", 3)]
    );

    Ok(())
}

#[test]
fn abstract_runs_are_interrupted_by_text_and_other_languages() -> Result<()> {
    for (separator, interruption_losses) in [
        (
            "Stray text",
            vec![("//TEI/teiHeader/profileDesc/text()", 1)],
        ),
        // Even an empty abstract, which is otherwise ignored
        (r#"<abstract xml:lang="fr"/>"#, vec![]),
    ] {
        let (article, losses) = decode_article(&tei(
            &format!(
                r#"<profileDesc xml:lang="en">
                    <abstract><p>First</p></abstract>
                    {separator}
                    <abstract><p>Second</p></abstract>
                </profileDesc>"#
            ),
            "",
            "",
        ))?;

        assert_eq!(
            article.r#abstract.as_ref().map(to_text).as_deref(),
            Some("First")
        );
        let expected: [(Option<String>, Vec<String>); 1] =
            [(Some("en".into()), vec!["Second".into()])];
        assert_eq!(abstract_parts(&article)?, expected);
        let mut expected_losses = vec![("//TEI/teiHeader/profileDesc/@lang", 1)];
        expected_losses.extend(interruption_losses);
        assert_eq!(losses.iter().collect::<Vec<_>>(), expected_losses);
    }

    Ok(())
}

#[test]
fn nonconsecutive_abstracts_in_same_language_stay_separate() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        r#"<profileDesc>
            <abstract xml:lang="en"><p>First</p></abstract>
            <textClass><keywords><term>Example</term></keywords></textClass>
            <abstract xml:lang="en"><p>Second</p></abstract>
        </profileDesc>"#,
        "",
        "",
    ))?;

    assert_eq!(
        article.r#abstract.as_ref().map(to_text).as_deref(),
        Some("First")
    );
    let expected: [(Option<String>, Vec<String>); 1] = [(Some("en".into()), vec!["Second".into()])];
    assert_eq!(abstract_parts(&article)?, expected);
    assert_eq!(article.options.keywords, Some(vec!["Example".into()]));
    assert_eq!(
        losses.iter().collect::<Vec<_>>(),
        [("//TEI/teiHeader/profileDesc/abstract/@lang", 1)]
    );

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

/// Check that internal links still resolve after decoding and exporting to JATS.
fn assert_jats_link_targets(article: Article, targets: &[&str]) -> Result<()> {
    let (jats, info) = stencila_codec_jats::encode(&Node::Article(article), None)?;
    let xml = roxmltree::Document::parse(&jats)?;
    for &target in targets {
        assert!(
            xml.descendants()
                .any(|node| node.attribute("id") == Some(target)),
            "missing anchor {target}: {jats}"
        );
        assert!(
            xml.descendants()
                .any(|node| node.has_tag_name("xref") && node.attribute("rid") == Some(target)),
            "missing cross-reference to {target}: {jats}"
        );
    }
    assert!(!labels(&info.losses).contains(&"Link.target"));
    Ok(())
}

#[test]
fn back_matter_division_ids_are_preserved() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r##"<p>See <ref target="#app1">Appendix A</ref>, <ref target="#app1a">its details</ref>, and <ref target="#app2">Appendix B</ref>.</p>"##,
        r#"<div type="annex">
            <div xml:id="app1"><head>Appendix A</head><p>Content.</p>
                <div xml:id="app1a"><head>Details</head><p>More content.</p></div>
            </div>
            <div id="app2"><head>Appendix B</head><p>Other content.</p></div>
        </div>"#,
    ))?;

    assert!(losses.is_empty(), "{:?}", labels(&losses));
    assert_jats_link_targets(article, &["app1", "app1a", "app2"])
}

#[test]
fn table_note_ids_are_preserved() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r##"<figure type="table" xml:id="tab_0">
            <table><row><cell>Value <ref type="foot" target="#tn1">a</ref> and <ref type="foot" target="#tn2">b</ref></cell></row></table>
            <note xml:id="tn1" place="foot"><p>First note.</p><p>Continued.</p></note>
            <note id="tn2" place="foot">Second note.</note>
        </figure>"##,
        "",
    ))?;

    // Table notes remain in the table rather than becoming article footnotes.
    let [Block::Table(table)] = article.content.as_slice() else {
        bail!("expected only a table")
    };
    assert!(table.notes.is_some());
    assert!(losses.is_empty(), "{:?}", labels(&losses));
    assert_jats_link_targets(article, &["tn1", "tn2"])
}

#[test]
fn table_note_blocks_are_preserved() -> Result<()> {
    let (article, losses) = decode_article(&tei(
        "",
        r#"<figure type="table">
            <table><row><cell>Value</cell></row></table>
            <note xml:id="tn1" place="foot">Before.
                <list><item>First item.</item><item>Second item.</item></list>
                <quote>Quoted text.</quote>
                After.
            </note>
        </figure>"#,
        "",
    ))?;

    let [Block::Table(table)] = article.content.as_slice() else {
        bail!("expected only a table")
    };
    let Some([Block::Paragraph(paragraph)]) = table.notes.as_deref() else {
        bail!("expected a paragraph containing the table note")
    };
    let [Inline::Note(note)] = paragraph.content.as_slice() else {
        bail!("expected an identified table note")
    };
    assert_eq!(note.id.as_deref(), Some("tn1"));

    // Block content is preserved even when the note has no explicit <p>.
    let [
        Block::Paragraph(before),
        Block::List(list),
        Block::QuoteBlock(quote),
        Block::Paragraph(after),
    ] = note.content.as_slice()
    else {
        bail!(
            "expected paragraphs, a list, and a quote: {:?}",
            note.content
        )
    };
    assert_eq!(to_text(before), "Before.");
    assert_eq!(
        list.items.iter().map(to_text).collect::<Vec<_>>(),
        ["First item.", "Second item."]
    );
    assert_eq!(to_text(quote), "Quoted text.");
    assert_eq!(to_text(after), "After.");
    assert!(losses.is_empty(), "{:?}", labels(&losses));

    Ok(())
}

#[test]
fn metadata_coordinates_and_xml_ids() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
        <author coords="1,10,20,30,40"><persName xml:id="person0" coords=" 1,1.00,2,3,4;2,5,6,7,8 "><forename>Jane</forename><surname>Doe</surname></persName>
            <affiliation xml:id="aff0" coords="1,10,11,12,13"><orgName coords="1,20,21,22,23">Institute</orgName><address coords="1,30,31,32,33"><country>NZ</country></address></affiliation>
        </author>
        <author xml:id="consortium" coords="2,1,2,3,4"><orgName coords="2,5,6,7,8">Consortium</orgName></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        r#"<listBibl><biblStruct xml:id="b0" coords="3,1,2,3,4;4,5,6,7,8"><monogr><title level="m">Book</title></monogr></biblStruct></listBibl>"#,
    ))?;
    let Some([Author::Person(person), Author::Organization(organization)]) =
        article.authors.as_deref()
    else {
        bail!("expected a person and an organization")
    };
    assert_eq!(person.id.as_deref(), Some("person0"));
    assert_eq!(
        boxes(&page_map, person)?,
        vec![
            (None, "1,10,20,30,40".into()),
            (Some("persName".into()), "1,1,2,3,4;2,5,6,7,8".into()),
        ]
    );
    // Coordinates are information about the source, not the person
    assert_eq!(person.options.identifiers, None);

    let Some([affiliation]) = person.affiliations.as_deref() else {
        bail!("expected one affiliation")
    };
    assert_eq!(affiliation.id.as_deref(), Some("aff0"));
    assert_eq!(
        boxes(&page_map, affiliation)?,
        vec![
            (None, "1,10,11,12,13".into()),
            (Some("orgName".into()), "1,20,21,22,23".into()),
            (Some("address".into()), "1,30,31,32,33".into()),
        ]
    );
    assert_eq!(organization.id.as_deref(), Some("consortium"));
    assert_eq!(
        boxes(&page_map, organization)?,
        vec![
            (None, "2,1,2,3,4".into()),
            (Some("orgName".into()), "2,5,6,7,8".into())
        ]
    );
    let Some([reference]) = article.references.as_deref() else {
        bail!("expected one reference")
    };
    assert_eq!(reference.id.as_deref(), Some("b0"));
    assert_eq!(
        boxes(&page_map, reference)?,
        vec![(None, "3,1,2,3,4;4,5,6,7,8".into())]
    );
    assert!(losses.is_empty(), "{:?}", labels(&losses));
    Ok(())
}

#[test]
fn content_coordinates_are_in_the_page_map() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        "",
        r#"<div><head xml:id="h0" coords="1,1,2,3,4">Heading</head>
        <p xml:id="p0" coords=" 1,2,3,4,5;2,6,7,8,9 ">Before<list><item>Item</item></list>After</p>
        <p coords="1,3,4,5,6">Another</p><p>No coordinates</p><p coords=" ">Empty coordinates</p>
        <formula xml:id="eq0" coords="2,1,2,3,4">x</formula>
    </div>"#,
        "",
    ))?;
    let [Block::Section(section)] = article.content.as_slice() else {
        bail!("expected a section")
    };
    let [
        Block::Heading(heading),
        Block::Paragraph(before),
        Block::List(_),
        Block::Paragraph(after),
        Block::Paragraph(another),
        Block::Paragraph(unlocated),
        Block::Paragraph(empty),
        Block::MathBlock(equation),
    ] = section.content.as_slice()
    else {
        bail!("unexpected section content: {:?}", section.content)
    };

    // Only ids in the TEI are kept, none are generated
    assert_eq!(heading.id.as_deref(), Some("h0"));
    assert_eq!(before.id.as_deref(), Some("p0"));
    assert_eq!(after.id, None);
    assert_eq!(another.id, None);
    assert_eq!(equation.id.as_deref(), Some("eq0"));

    let located = |boxes: &str| vec![(None, boxes.to_string())];
    assert_eq!(boxes(&page_map, heading)?, located("1,1,2,3,4"));
    // Both parts of a paragraph split around a list are where the paragraph is
    assert_eq!(boxes(&page_map, before)?, located("1,2,3,4,5;2,6,7,8,9"));
    assert_eq!(boxes(&page_map, after)?, located("1,2,3,4,5;2,6,7,8,9"));
    assert_eq!(boxes(&page_map, another)?, located("1,3,4,5,6"));
    assert_eq!(boxes(&page_map, unlocated)?, vec![]);
    assert_eq!(boxes(&page_map, empty)?, vec![]);
    assert_eq!(boxes(&page_map, equation)?, located("2,1,2,3,4"));
    assert_eq!(page_map.entries().len(), 5);

    assert_eq!(article.options.identifiers, None);
    assert_eq!(
        labels(&losses),
        vec!["//TEI/text/body/div/formula (text, not TeX or MathML)"]
    );
    Ok(())
}

#[test]
fn malformed_coordinates_are_a_loss() -> Result<()> {
    let (_, losses, page_map) = decode_article_with_page_map(&tei(
        "",
        r#"<div><head>Heading</head><p coords="1,2,3">Text</p></div>"#,
        "",
    ))?;
    assert!(page_map.is_empty());
    assert_eq!(
        labels(&losses),
        vec!["//TEI/text/body/div/p/@coords (malformed)"]
    );
    Ok(())
}

#[test]
fn coordinates_of_dropped_nodes_are_not_mapped() -> Result<()> {
    let (_, _, page_map) = decode_article_with_page_map(&tei(
        r#"<profileDesc><abstract><div><head coords="1,1,1,1,1">Abstract</head><p coords="1,2,2,2,2">Text</p></div></abstract></profileDesc>"#,
        r#"<div><head coords="2,1,1,1,1">Heading</head></div><note place="foot" xml:id="foot_0" coords="3,1,1,1,1"> </note>"#,
        "",
    ))?;
    // Not the abstract's heading, which is dropped, nor the empty footnote
    let pages = page_map
        .entries()
        .iter()
        .flat_map(|entry| entry.boxes.iter().map(|page_box| page_box.page))
        .collect::<Vec<_>>();
    assert_eq!(pages, vec![1, 2]);
    Ok(())
}

#[test]
fn ids_and_coordinates_of_dropped_nodes_are_a_loss() -> Result<()> {
    let (_, losses, _) = decode_article_with_page_map(&tei(
        r#"<profileDesc><abstract><div coords="1,9,9,9,9"><head xml:id="h0" coords="1,1,1,1,1">Abstract</head><p>Text</p></div></abstract></profileDesc>"#,
        r#"<div><head>Heading</head></div><note place="foot" xml:id="foot_0" coords="3,1,1,1,1"> </note>"#,
        "",
    ))?;
    let labels = labels(&losses);
    for label in [
        "//TEI/teiHeader/profileDesc/abstract/div/@coords",
        "//TEI/teiHeader/profileDesc/abstract/div/head/@id",
        "//TEI/teiHeader/profileDesc/abstract/div/head/@coords",
        "//TEI/text/body/note/@id",
        "//TEI/text/body/note/@coords",
    ] {
        assert!(labels.contains(&label), "{label} not in {labels:?}");
    }
    Ok(())
}

#[test]
fn paragraphs_of_only_blocks_lose_their_id_and_coordinates() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        "",
        r#"<div><head>Heading</head><p xml:id="p0" coords="1,1,1,1,1"><figure xml:id="fig_0"><head>Figure</head></figure></p><p xml:id="p1" coords="2,1,1,1,1"> </p></div>"#,
        "",
    ))?;
    let [Block::Section(section)] = article.content.as_slice() else {
        bail!("expected a section: {:?}", article.content)
    };
    let [Block::Heading(..), Block::Figure(..)] = section.content.as_slice() else {
        bail!("expected a heading and a figure: {:?}", section.content)
    };
    assert!(page_map.is_empty());

    // Both paragraphs lose them
    let counts = losses
        .iter()
        .filter(|(label, ..)| label.starts_with("//TEI/text/body/div/p/@"))
        .collect::<Vec<_>>();
    assert_eq!(
        counts,
        vec![
            ("//TEI/text/body/div/p/@coords", 2),
            ("//TEI/text/body/div/p/@id", 2)
        ]
    );
    Ok(())
}

#[test]
fn empty_person_names_lose_their_id_and_coordinates() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
        <author><persName xml:id="pn0" coords="1,1,1,1,1"/><orgName coords="1,2,2,2,2">Consortium</orgName></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;
    let Some([Author::Organization(organization)]) = article.authors.as_deref() else {
        bail!("expected an organization: {:?}", article.authors)
    };
    assert_eq!(organization.id, None);
    assert_eq!(
        boxes(&page_map, organization)?,
        vec![(Some("orgName".to_string()), "1,2,2,2,2".to_string())]
    );

    let labels = labels(&losses);
    for label in [
        "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/persName/@id",
        "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/persName/@coords",
    ] {
        assert!(labels.contains(&label), "{label} not in {labels:?}");
    }
    Ok(())
}

#[test]
fn coordinates_out_of_range_are_malformed() -> Result<()> {
    for coords in [
        "1,NaN,1,1,1",
        "1,1,inf,1,1",
        "0,1,1,1,1",
        "1,1,1,-1,1",
        "1,1,1,1,-1",
    ] {
        let (_, losses, page_map) = decode_article_with_page_map(&tei(
            "",
            &format!(r#"<div><head>Heading</head><p coords="{coords}">Text</p></div>"#),
            "",
        ))?;
        assert!(page_map.is_empty(), "{coords}");
        assert_eq!(
            labels(&losses),
            vec!["//TEI/text/body/div/p/@coords (malformed)"],
            "{coords}"
        );
    }
    Ok(())
}

#[test]
fn coordinates_do_not_change_structure() -> Result<()> {
    let decode = |coords: &str| {
        decode_article(&tei(
            &format!(
                r#"<profileDesc><abstract><div{coords}><head>Abstract</head><p>Hello</p></div></abstract></profileDesc>"#
            ),
            "",
            "",
        ))
    };
    let (plain, ..) = decode("")?;
    let (located, ..) = decode(r#" coords="1,1,1,1,1""#)?;
    assert_eq!(plain, located);
    let Some([Block::Paragraph(..)]) = located.r#abstract.as_deref() else {
        bail!("expected a paragraph: {:?}", located.r#abstract)
    };
    Ok(())
}

#[test]
fn footnote_marker_and_content_are_parts_of_the_note() -> Result<()> {
    let (article, _, page_map) = decode_article_with_page_map(&tei(
        "",
        r##"<div><head>Heading</head><p>A claim<ref type="foot" target="#foot_0" coords="1,1,1,1,1">1</ref>.</p></div>
        <note place="foot" xml:id="foot_0" coords="9,1,1,1,1"><p>The footnote.</p></note>"##,
        "",
    ))?;
    let [Block::Section(section)] = article.content.as_slice() else {
        bail!("expected a section")
    };
    let Some(Block::Paragraph(paragraph)) = section.content.get(1) else {
        bail!("expected a paragraph")
    };
    let Some(Inline::Note(note)) = paragraph.content.get(1) else {
        bail!("expected a note: {:?}", paragraph.content)
    };
    assert_eq!(
        boxes(&page_map, note)?,
        vec![
            (None, "9,1,1,1,1".into()),
            (Some("marker".into()), "1,1,1,1,1".into())
        ]
    );
    Ok(())
}

#[test]
fn coordinates_do_not_change_alignment() -> Result<()> {
    let decode = |body: &str| -> Result<Node> { Ok(decode(&tei("", body, ""), None)?.0) };
    let left =
        decode(r#"<p coords="1,1,1,1,1">Alpha beta.</p><p coords="1,2,2,2,2">Gamma delta.</p>"#)?;
    let right = decode(
        r#"<p coords="1,1,1,1,1">Inserted text.</p><p coords="1,2,2,2,2">Alpha beta.</p><p coords="1,3,3,3,3">Gamma delta.</p>"#,
    )?;
    let comparison = stencila_node_compare::compare(&left, &right)?;
    let pairs = comparison
        .alignment()
        .pairs()
        .map(|(left, right, _)| (left.path.to_string(), right.path.to_string()))
        .collect::<Vec<_>>();
    for pair in [("content/0", "content/1"), ("content/1", "content/2")] {
        assert!(
            pairs.contains(&(pair.0.to_string(), pair.1.to_string())),
            "{pair:?} not in {pairs:?}"
        );
    }
    Ok(())
}

#[test]
fn page_coordinates_do_not_create_jats_comparison_differences() -> Result<()> {
    let (jats, _) = stencila_codec_jats::decode(
        r#"<article><front><article-meta>
        <title-group><article-title>Test</article-title></title-group>
        <contrib-group><contrib contrib-type="author"><name><surname>Doe</surname><given-names>Jane</given-names></name><xref ref-type="aff" rid="aff0"/></contrib></contrib-group>
        <aff id="aff0"><institution>Institute</institution></aff>
        </article-meta></front><body><sec><title>Heading</title><p>Text.<xref ref-type="bibr" rid="b0">[1]</xref></p></sec></body>
        <back><ref-list><ref id="b0"><element-citation><source>Book</source></element-citation></ref></ref-list></back>
        </article>"#,
        None,
    )?;
    let (located, _) = decode(
        &tei(
            r#"<fileDesc><titleStmt><title>Test</title></titleStmt><sourceDesc><biblStruct><analytic>
        <author><persName coords="1,1,2,3,4"><forename>Jane</forename><surname>Doe</surname></persName><affiliation xml:id="aff0" coords="1,2,3,4,5"><orgName>Institute</orgName></affiliation></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
            r##"<div><head coords="1,3,4,5,6">Heading</head><p coords="1,4,5,6,7">Text.<ref xml:id="cite0" coords="1,7,8,9,10" type="bibr" target="#b0">[1]</ref></p></div>"##,
            r#"<listBibl><biblStruct xml:id="b0" coords="1,5,6,7,8"><monogr><title level="m">Book</title></monogr></biblStruct></listBibl>"#,
        ),
        None,
    )?;
    for (left, right) in [(&jats, &located), (&located, &jats)] {
        let comparison = stencila_node_compare::compare(left, right)?;
        assert!(comparison.is_equal(), "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn source_metadata_without_a_node_to_carry_it_is_a_loss() -> Result<()> {
    let (_, losses) = decode_article(&tei(
        "",
        r##"<div><head>Introduction</head><p>See<ptr xml:id="x" coords="1,2,3,4,5"/>. A claim<ref xml:id="r1" type="foot" target="#foot_0">1</ref>.</p></div>
        <div xml:id="d1" coords="1,1,1,1,1"> </div>
        <note place="foot" xml:id="foot_0"><p>The footnote.</p></note>"##,
        "",
    ))?;
    let labels = labels(&losses);
    for label in [
        "//TEI/text/body/div/p/ptr/@id",
        "//TEI/text/body/div/p/ptr/@coords",
        "//TEI/text/body/div/p/ref/@id",
        "//TEI/text/body/div/@id",
        "//TEI/text/body/div/@coords",
    ] {
        assert!(labels.contains(&label), "{label} not in {labels:?}");
    }
    Ok(())
}

#[test]
fn author_ids_are_from_the_author_or_its_name() -> Result<()> {
    let (article, losses, page_map) = decode_article_with_page_map(&tei(
        r#"<fileDesc><sourceDesc><biblStruct><analytic>
        <author xml:id="a1"><persName xml:id="p1"><surname>Doe</surname></persName></author>
        <author><orgName xml:id="o1" coords="1,1,1,1,1">Institute</orgName><persName xml:id="p2"><surname>Roe</surname></persName></author>
        <author xml:id="a3" coords="1,2,3,4,5"><email>x@example.org</email></author>
        </analytic></biblStruct></sourceDesc></fileDesc>"#,
        "",
        "",
    ))?;
    let Some([Author::Person(doe), Author::Person(roe)]) = article.authors.as_deref() else {
        bail!("expected two people: {:?}", article.authors)
    };
    assert_eq!(doe.id.as_deref(), Some("a1"));

    // The `orgName` of a person is lost, along with its id and coordinates
    assert_eq!(roe.id.as_deref(), Some("p2"));
    assert_eq!(boxes(&page_map, roe)?, vec![]);

    let labels = labels(&losses);
    for label in [
        "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/persName/@id",
        "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author/orgName",
        "//TEI/teiHeader/fileDesc/sourceDesc/biblStruct/analytic/author",
    ] {
        assert!(labels.contains(&label), "{label} not in {labels:?}");
    }
    Ok(())
}
