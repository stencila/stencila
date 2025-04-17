use common::{
    eyre::{bail, Result},
    tokio,
};
use common_dev::pretty_assertions::assert_eq;
use node_canonicalize::canonicalize;
use schema::{Node, Organization};

/// Organization with ROR on OpenAlex should get ROR
#[tokio::test]
async fn open_alex_ror() -> Result<()> {
    let mut org = Node::Organization(Organization {
        name: Some("Brown University".into()),
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, "05gq02987")
    } else {
        bail!("No ROR")
    };

    Ok(())
}

/// Organization with no ROR on OpenAlex should get ROR
/// derived from OpenAlex ID.
#[tokio::test]
async fn open_alex_id() -> Result<()> {
    let mut org = Node::Organization(Organization {
        name: Some("Deleted Institution".into()),
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, "O4389424196")
    } else {
        bail!("No ROR")
    };

    Ok(())
}

/// Organization with no name will always fallback to having
/// its ROR derived from CBOR hash
#[tokio::test]
async fn cbor_hash() -> Result<()> {
    let mut org = Node::Organization(Organization {
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, "Sddx6tq37")
    } else {
        bail!("No ROR")
    };

    Ok(())
}
