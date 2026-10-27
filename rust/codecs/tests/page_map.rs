//! Tests of the page map of decoded documents

use std::io::Write;

use pretty_assertions::assert_eq;

use stencila_codec::eyre::{Result, bail};
use stencila_codecs::{
    from_path_with_info,
    stencila_schema::{Block, Inline, Node},
};

/// Citation groups that are merged when citations are normalized keep the
/// boxes of each of them
#[tokio::test]
async fn merged_citation_groups_keep_their_boxes() -> Result<()> {
    let mut file = tempfile::Builder::new().suffix(".tei.xml").tempfile()?;
    file.write_all(
        br##"<?xml version="1.0" encoding="UTF-8"?>
<TEI xmlns="http://www.tei-c.org/ns/1.0">
<teiHeader/>
<text><body><div><head>Heading</head><p>A claim <ref type="bibr" target="#b0 #b1" coords="1,1,1,1,1">1,2</ref>, <ref type="bibr" target="#b2 #b3" coords="2,2,2,2,2">3,4</ref>.</p></div></body>
<back><listBibl>
<biblStruct xml:id="b0"><analytic><title>Zero</title></analytic></biblStruct>
<biblStruct xml:id="b1"><analytic><title>One</title></analytic></biblStruct>
<biblStruct xml:id="b2"><analytic><title>Two</title></analytic></biblStruct>
<biblStruct xml:id="b3"><analytic><title>Three</title></analytic></biblStruct>
</listBibl></back></text>
</TEI>"##,
    )?;

    let (node, _, info) = from_path_with_info(file.path(), None).await?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    let Some(Block::Section(section)) = article.content.first() else {
        bail!("expected a section: {:?}", article.content)
    };
    let Some(Block::Paragraph(paragraph)) = section.content.get(1) else {
        bail!("expected a paragraph: {:?}", section.content)
    };
    let groups = paragraph
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::CitationGroup(group) => Some(group),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [group] = groups.as_slice() else {
        bail!("expected one citation group: {:?}", paragraph.content)
    };
    assert_eq!(group.items.len(), 4);

    let pages = info
        .page_map
        .entries_for(&group.node_id())
        .flat_map(|entry| entry.boxes.iter().map(|page_box| page_box.page))
        .collect::<Vec<_>>();
    assert_eq!(pages, vec![1, 2]);

    Ok(())
}
