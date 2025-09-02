use eyre::{Result, bail};
use serde_json::json;

use node_canonicalize::canonicalize;
use pretty_assertions::assert_eq;
use schema::{Article, Author, Date, Node, Person, shortcuts::t};

/// Article with a DOI on OpenAlex should get that DOI
#[tokio::test]
async fn open_alex_doi() -> Result<()> {
    let mut article = Node::Article(Article {
        title: Some(vec![t(
            "Effect of modified zirconium oxide nano-fillers addition on some properties of heat cure acrylic denture base material",
        )]),
        date_published: Some(Date::new("2012".into())),
        ..Default::default()
    });
    canonicalize(&mut article).await?;

    if let Node::Article(Article { doi: Some(doi), .. }) = article {
        assert_eq!(doi, "10.0001/1318")
    } else {
        bail!("No DOI")
    };

    Ok(())
}

/// Article with no DOI on OpenAlex should get DOI generated
/// from OpenAlex id
#[tokio::test]
async fn open_alex_id() -> Result<()> {
    let mut article = Node::Article(Article {
        title: Some(vec![t(
            "R: A language and environment for statistical computing",
        )]),
        ..Default::default()
    });
    canonicalize(&mut article).await?;

    if let Node::Article(Article { doi: Some(doi), .. }) = article {
        assert_eq!(doi, "10.0000/openalex.W2582743722")
    } else {
        bail!("No DOI")
    };

    Ok(())
}

/// Article with no title will always fallback to having
/// its ROR derived from CBOR hash
#[tokio::test]
async fn cbor_hash() -> Result<()> {
    let mut article = Node::Article(Article {
        ..Default::default()
    });
    canonicalize(&mut article).await?;

    if let Node::Article(Article { doi: Some(doi), .. }) = article {
        assert_eq!(doi, "10.0000/stencila.wK_lgMM7hCM")
    } else {
        bail!("No DOI")
    };

    Ok(())
}

/// Article with authors having affiliations should have RORs canonicalized
/// even if they have an ORCID
#[tokio::test]
async fn open_alex_affiliations() -> Result<()> {
    let mut article = serde_json::from_value(json!({
        "type": "Article",
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
        }],
        "content": []
    }))?;
    canonicalize(&mut article).await?;

    if let Node::Article(Article {
        authors: Some(authors),
        ..
    }) = article
    {
        if let Some(Author::Person(Person { affiliations, .. })) = authors.first() {
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
