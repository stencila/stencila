use common::{
    eyre::{bail, Result},
    tokio,
};
use common_dev::pretty_assertions::assert_eq;
use node_canonicalize::canonicalize;
use schema::{Node, Reference};

/// Reference with no title will always fallback to having
/// its ROR derived from CBOR hash
#[tokio::test]
async fn cbor_hash() -> Result<()> {
    let mut reference = Node::Reference(Reference {
        ..Default::default()
    });
    canonicalize(&mut reference).await?;

    if let Node::Reference(Reference {
        doi: Some(doi), ..
    }) = reference
    {
        assert_eq!(doi, "10.0000/stencila.aOoQvBTTtbA")
    } else {
        bail!("No DOI")
    };

    Ok(())
}
