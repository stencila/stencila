use common::{
    eyre::{bail, Result},
    tokio,
};
use common_dev::pretty_assertions::assert_eq;
use node_canonicalize::canonicalize;
use schema::{Node, Person};

/// Person with ORCID on OpenAlex should get ORCID
#[tokio::test]
async fn open_alex_orcid() -> Result<()> {
    let mut person = Node::Person(Person {
        family_names: Some(vec!["Carberry".into()]),
        given_names: Some(vec!["Josiah".into()]),
        ..Default::default()
    });
    canonicalize(&mut person).await?;

    if let Node::Person(Person {
        orcid: Some(orcid), ..
    }) = person
    {
        assert_eq!(orcid, "0000-0002-1825-0097")
    } else {
        bail!("No ORCID")
    };

    Ok(())
}

/// Person with no ORCID on OpenAlex should get ORCID
/// derived from OpenAlex ID.
#[tokio::test]
async fn open_alex_id() -> Result<()> {
    let mut person = Node::Person(Person {
        family_names: Some(vec!["Einstein".into()]),
        given_names: Some(vec!["Albert".into()]),
        ..Default::default()
    });
    canonicalize(&mut person).await?;

    if let Node::Person(Person {
        orcid: Some(orcid), ..
    }) = person
    {
        assert_eq!(orcid, "A000-0050-8313-8872")
    } else {
        bail!("No ORCID")
    };

    Ok(())
}

/// Person with no name will always fallback to having their
/// ORCID derived from CBOR hash
#[tokio::test]
async fn cbor_hash() -> Result<()> {
    let mut person = Node::Person(Person {
        ..Default::default()
    });
    canonicalize(&mut person).await?;

    if let Node::Person(Person {
        orcid: Some(orcid), ..
    }) = person
    {
        assert_eq!(orcid, "S327-4486-9489-6164")
    } else {
        bail!("No ORCID")
    };

    Ok(())
}
