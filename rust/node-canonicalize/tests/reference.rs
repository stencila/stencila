use common::{
    eyre::{bail, Result},
    tokio,
};
use common_dev::pretty_assertions::assert_eq;
use node_canonicalize::canonicalize;
use schema::{shortcuts::t, Date, Node, Reference};

/// Reference with a DOI on OpenAlex should get that DOI
#[tokio::test]
async fn open_alex_doi() -> Result<()> {
    let mut reference = Node::Reference(Reference {
        title: Some(vec![t(
            "Effect of modified zirconium oxide nano-fillers addition on some properties of heat cure acrylic denture base material",
        )]),
        date: Some(Date::new("2012".into())),
        ..Default::default()
    });
    canonicalize(&mut reference).await?;

    if let Node::Reference(Reference { doi: Some(doi), .. }) = reference {
        assert_eq!(doi, "10.0001/1318")
    } else {
        bail!("No DOI")
    };

    Ok(())
}

/// Reference with no DOI on OpenAlex should get DOI generated
/// from OpenAlex id
#[tokio::test]
async fn open_alex_id() -> Result<()> {
    let mut reference = Node::Reference(Reference {
        title: Some(vec![t(
            "R: A language and environment for statistical computing",
        )]),
        ..Default::default()
    });
    canonicalize(&mut reference).await?;

    if let Node::Reference(Reference { doi: Some(doi), .. }) = reference {
        assert_eq!(doi, "10.0000/openalex.W2582743722")
    } else {
        bail!("No DOI")
    };

    Ok(())
}

/// Reference with no title will always fallback to having
/// its ROR derived from CBOR hash
#[tokio::test]
async fn cbor_hash() -> Result<()> {
    let mut reference = Node::Reference(Reference {
        ..Default::default()
    });
    canonicalize(&mut reference).await?;

    if let Node::Reference(Reference { doi: Some(doi), .. }) = reference {
        assert_eq!(doi, "10.0000/stencila.aOoQvBTTtbA")
    } else {
        bail!("No DOI")
    };

    Ok(())
}
