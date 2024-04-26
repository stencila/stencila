use codec::{
    common::{
        eyre::{Ok, Result},
        tokio,
    },
    Codec, EncodeInfo, NodeType, PoshMap, Position16, Position8, Range16, Range8,
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::assert_snapshot;

#[tokio::test]
async fn same() -> Result<()> {
    let codec = MarkdownCodec {};

    let source = "para1\n\npara2\n";
    let (node, ..) = codec.from_str(source, None).await?;
    let (generated, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_eq!(generated, source);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
         0      5         (0, 5)   Text.value
         0      5         (0, 0)   Text
         0      5         (0, 0)   Paragraph.content
         0      6         (0, 1)   Paragraph
         7     12         (7, 6)   Text.value
         7     12         (0, 0)   Text
         7     12         (0, 0)   Paragraph.content
         7     13         (0, 1)   Paragraph
         0     14        (-7, 1)   Article.content
         0     14         (0, 0)   Article
    "###);

    let p1 = mapping.entry_at(5).unwrap().clone();
    assert_eq!(p1.node_type, NodeType::Paragraph);
    let p2 = mapping.entry_at(12).unwrap().clone();
    assert_eq!(p2.node_type, NodeType::Paragraph);
    let art = mapping.entry_at(13).unwrap().clone();
    assert_eq!(art.node_type, NodeType::Article);

    let poshmap = PoshMap::new(&source, &generated, mapping);

    assert_eq!(
        poshmap.node_id_to_range8(&p1.node_id),
        Some(Range8::new(Position8::new(0, 0), Position8::new(0, 5)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(0, 5)),
        Some(&p1.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range8(&p2.node_id),
        Some(Range8::new(Position8::new(2, 0), Position8::new(2, 5)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(2, 5)),
        Some(&p2.node_id)
    );

    Ok(())
}

#[tokio::test]
async fn spacing() -> Result<()> {
    let codec = MarkdownCodec {};

    let source = "\npara1\n\n\n\npara2\n\n";
    let (node, ..) = codec.from_str(source, None).await?;
    let (generated, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_eq!(generated, "para1\n\npara2\n");

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
         0      5         (0, 5)   Text.value
         0      5         (0, 0)   Text
         0      5         (0, 0)   Paragraph.content
         0      6         (0, 1)   Paragraph
         7     12         (7, 6)   Text.value
         7     12         (0, 0)   Text
         7     12         (0, 0)   Paragraph.content
         7     13         (0, 1)   Paragraph
         0     14        (-7, 1)   Article.content
         0     14         (0, 0)   Article
    "###);

    let p1 = mapping.entry_at(5).unwrap().clone();
    assert_eq!(p1.node_type, NodeType::Paragraph);
    let p2 = mapping.entry_at(12).unwrap().clone();
    assert_eq!(p2.node_type, NodeType::Paragraph);
    let art = mapping.entry_at(13).unwrap().clone();
    assert_eq!(art.node_type, NodeType::Article);

    let poshmap = PoshMap::new(&source, &generated, mapping);

    assert_eq!(
        poshmap.node_id_to_range8(&p1.node_id),
        Some(Range8::new(Position8::new(1, 0), Position8::new(1, 5)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(1, 5)),
        Some(&p1.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range16(&p1.node_id),
        Some(Range16::new(Position16::new(1, 0), Position16::new(1, 5)))
    );
    assert_eq!(
        poshmap.position16_to_node_id(Position16::new(1, 5)),
        Some(&p1.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range8(&p2.node_id),
        Some(Range8::new(Position8::new(5, 0), Position8::new(5, 5)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(5, 5)),
        Some(&p2.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range16(&p2.node_id),
        Some(Range16::new(Position16::new(5, 0), Position16::new(5, 5)))
    );
    assert_eq!(
        poshmap.position16_to_node_id(Position16::new(5, 5)),
        Some(&p2.node_id)
    );

    Ok(())
}

#[tokio::test]
async fn emoji() -> Result<()> {
    let codec = MarkdownCodec {};

    let source = "para😊1\n\np😊ara2\n";
    let (node, ..) = codec.from_str(source, None).await?;
    let (generated, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_eq!(generated, source);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
         0      6         (0, 6)   Text.value
         0      6         (0, 0)   Text
         0      6         (0, 0)   Paragraph.content
         0      7         (0, 1)   Paragraph
         8     14         (8, 7)   Text.value
         8     14         (0, 0)   Text
         8     14         (0, 0)   Paragraph.content
         8     15         (0, 1)   Paragraph
         0     16        (-8, 1)   Article.content
         0     16         (0, 0)   Article
    "###);

    let p1 = mapping.entry_at(6).unwrap().clone();
    assert_eq!(p1.node_type, NodeType::Paragraph);
    let p2 = mapping.entry_at(14).unwrap().clone();
    assert_eq!(p2.node_type, NodeType::Paragraph);
    let art = mapping.entry_at(15).unwrap().clone();
    assert_eq!(art.node_type, NodeType::Article);

    let poshmap = PoshMap::new(&source, &generated, mapping);

    assert_eq!(
        poshmap.node_id_to_range8(&p1.node_id),
        Some(Range8::new(Position8::new(0, 0), Position8::new(0, 6)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(0, 6)),
        Some(&p1.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range16(&p1.node_id),
        Some(Range16::new(Position16::new(0, 0), Position16::new(0, 7)))
    );
    assert_eq!(
        poshmap.position16_to_node_id(Position16::new(0, 7)),
        Some(&p1.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range8(&p2.node_id),
        Some(Range8::new(Position8::new(2, 0), Position8::new(2, 6)))
    );
    assert_eq!(
        poshmap.position8_to_node_id(Position8::new(2, 6)),
        Some(&p2.node_id)
    );

    assert_eq!(
        poshmap.node_id_to_range16(&p2.node_id),
        Some(Range16::new(Position16::new(2, 0), Position16::new(2, 7)))
    );
    assert_eq!(
        poshmap.position16_to_node_id(Position16::new(2, 7)),
        Some(&p2.node_id)
    );

    Ok(())
}
