use eyre::{Result, bail};

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

/// Organization name with departments should get ROR
#[tokio::test]
async fn open_alex_ror_departments() -> Result<()> {
    const NIH_ROR: &str = "01cwqze88";

    let mut org = Node::Organization(Organization {
        name: Some("National Institutes of Health".into()),
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, NIH_ROR)
    } else {
        bail!("No ROR")
    };

    let mut org = Node::Organization(Organization {
        name: Some("National Heart Lung and Blood Institute, National Institutes of Health".into()),
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, NIH_ROR)
    } else {
        bail!("No ROR")
    };

    let mut org = Node::Organization(Organization {
        name: Some("Cell and Developmental Biology Center, National Heart Lung and Blood Institute, National Institutes of Health".into()),
        ..Default::default()
    });
    canonicalize(&mut org).await?;

    if let Node::Organization(Organization { ror: Some(ror), .. }) = org {
        assert_eq!(ror, NIH_ROR)
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
