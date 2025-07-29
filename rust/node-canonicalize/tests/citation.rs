use common::{eyre::Result, tokio};
use common_dev::insta::assert_json_snapshot;
use node_canonicalize::canonicalize;
use schema::{
    Article, Node, Reference,
    shortcuts::{ct, p},
};

/// Citations are canonicalized by using the DOI of the cited reference
/// as the target (and by cloning reference to `cites`)
#[tokio::test]
async fn basic() -> Result<()> {
    let mut article = Node::Article(Article {
        content: vec![p(vec![ct("ref1")])],
        references: Some(vec![Reference {
            id: Some("ref1".into()),
            doi: Some("10.7717/peerj.4375".into()),
            ..Default::default()
        }]),
        ..Default::default()
    });
    canonicalize(&mut article).await?;

    assert_json_snapshot!(article, @r#"
    {
      "type": "Article",
      "doi": "10.0000/stencila.aOoQvBTTtbA",
      "references": [
        {
          "type": "Reference",
          "id": "ref1",
          "doi": "10.7717/peerj.4375"
        }
      ],
      "content": [
        {
          "type": "Paragraph",
          "content": [
            {
              "type": "Citation",
              "target": "10.7717/peerj.4375",
              "cites": {
                "type": "Reference",
                "id": "ref1",
                "doi": "10.7717/peerj.4375"
              }
            }
          ]
        }
      ]
    }
    "#);

    Ok(())
}
