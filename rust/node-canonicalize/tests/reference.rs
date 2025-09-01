use eyre::{Result, bail};
use serde_json::json;

use common_dev::pretty_assertions::assert_eq;
use node_canonicalize::canonicalize;
use schema::{Author, Date, Node, Person, Reference, shortcuts::t};

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

/// Reference with authors having affiliations should have RORs canonicalized
/// even if they have an ORCID
#[tokio::test]
async fn open_alex_affiliations() -> Result<()> {
    let mut reference = serde_json::from_value(json!({
        "type": "Reference",
        "doi": "10.1101/2023.12.31.573522",
        "authors": [{
            "type": "Person",
            "orcid": "0000-0002-6935-0047",
            "familyNames": ["Penker"],
            "givenNames": ["Sapir"],
            "affiliations": [{
                "type": "Organization",
                "name": "Department of Medical Neurobiology, Faculty of Medicine and IMRIC, The Hebrew University of Jerusalem",
            }],
        }]
    }))?;
    canonicalize(&mut reference).await?;

    if let Node::Reference(Reference {
        authors: Some(authors),
        ..
    }) = reference
    {
        if let Some(Author::Person(Person {
            orcid,
            affiliations,
            ..
        })) = authors.first()
        {
            assert_eq!(orcid.as_deref(), Some("0000-0002-6935-0047"));
            assert_eq!(
                affiliations
                    .iter()
                    .flatten()
                    .next()
                    .and_then(|org| org.ror.as_deref()),
                Some("03qxff017")
            );
        } else {
            bail!("Not a person")
        }
    } else {
        bail!("No authors!")
    };

    Ok(())
}
