use std::sync::LazyLock;

use eyre::Result;
use futures::future::try_join_all;
use regex::Regex;

use schema::{
    Article, Author, AuthorRole, AuthorRoleAuthor, Citation, Node, Organization, Person, Reference,
    VisitorMut, WalkControl, WalkNode, replicate,
};

mod cbor_hash;
mod openalex;

/// Canonicalize a node
///
/// Sets the canonical id for `Article` and `Reference` nodes (DOI),
/// `Person` nodes (ORCID), and `Organization` nodes (ROR).
pub async fn canonicalize(node: &mut Node) -> Result<()> {
    node.canonicalize().await?;

    let mut linker = Linker::new(node);
    node.walk_mut(&mut linker);

    Ok(())
}

/// Is an optional id a valid DOI
fn is_doi(id: &Option<String>) -> bool {
    let Some(id) = id else { return false };

    if id.starts_with("10.0000/") {
        return false;
    }

    static REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)^10.\d{4,9}/[-._;()/:A-Z0-9]+$").expect("invalid regex"));
    REGEX.is_match(id)
}

/// Is an optional id a valid ORCID
fn is_orcid(id: &Option<String>) -> bool {
    let Some(id) = id else { return false };
    static REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\d{4}-\d{4}-\d{4}-\d{3}[0-9X]$").expect("invalid regex"));
    REGEX.is_match(id)
}

/// Is an optional id a valid ROR
fn is_ror(id: &Option<String>) -> bool {
    let Some(id) = id else { return false };
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$").expect("invalid regex")
    });
    REGEX.is_match(id)
}

#[derive(Default)]
struct Linker {
    references: Option<Vec<Reference>>,
}

impl Linker {
    fn new(node: &Node) -> Self {
        let references = match node {
            Node::Article(node) => node.references.clone(),
            _ => None,
        };

        Self { references }
    }
}

impl VisitorMut for Linker {
    fn visit_citation(&mut self, citation: &mut Citation) -> WalkControl {
        // Do not change the targets of citations that are already DOIs and which have a reference
        if citation.target.starts_with("10.") && citation.options.cites.is_some() {
            return WalkControl::Continue;
        }

        // Find the reference with the target as either id or DOI
        if let Some(reference) = self.references.iter().flatten().find(|reference| {
            reference.id.as_ref() == Some(&citation.target)
                || reference.doi.as_ref() == Some(&citation.target)
        }) {
            if let Some(doi) = &reference.doi {
                citation.target = doi.clone();
            }
            if let Ok(cites) = replicate(reference) {
                citation.options.cites = Some(cites);
            }
        };

        WalkControl::Continue
    }
}

trait Canonicalize {
    async fn canonicalize(&mut self) -> Result<()>;
}

impl Canonicalize for Node {
    async fn canonicalize(&mut self) -> Result<()> {
        match self {
            Node::Article(node) => node.canonicalize().await,

            // These node types are not normally canonicalized directly but are included
            // here primarily for tests
            Node::Organization(node) => node.canonicalize().await,
            Node::Person(node) => node.canonicalize().await,
            Node::Reference(node) => node.canonicalize().await,

            _ => Ok(()),
        }
    }
}

impl Canonicalize for Article {
    async fn canonicalize(&mut self) -> Result<()> {
        // Generate a reference for the article, canonicalize that and then update ids
        // of parts including ORCIDs of authors and RORs of their affiliations.

        let mut reference = Reference::from(&*self);
        reference.canonicalize().await?;

        if !is_doi(&self.doi) {
            self.doi = reference.doi;
        }

        // This will canonicalize ids based on authorship of the article, which
        // is preferable to just based on names.
        for (author1, author2) in self
            .authors
            .iter_mut()
            .flatten()
            .zip(reference.authors.into_iter().flatten())
        {
            match (author1, author2) {
                (Author::Person(person1), Author::Person(person2))
                | (
                    Author::AuthorRole(AuthorRole {
                        author: AuthorRoleAuthor::Person(person1),
                        ..
                    }),
                    Author::AuthorRole(AuthorRole {
                        author: AuthorRoleAuthor::Person(person2),
                        ..
                    }),
                ) => {
                    person1.orcid = person2.orcid;

                    for (aff1, aff2) in person1
                        .affiliations
                        .iter_mut()
                        .flatten()
                        .zip(person2.affiliations.into_iter().flatten())
                    {
                        aff1.ror = aff2.ror;
                    }
                }
                (Author::Organization(org1), Author::Organization(org2))
                | (
                    Author::AuthorRole(AuthorRole {
                        author: AuthorRoleAuthor::Organization(org1),
                        ..
                    }),
                    Author::AuthorRole(AuthorRole {
                        author: AuthorRoleAuthor::Organization(org2),
                        ..
                    }),
                ) => {
                    org1.ror = org2.ror;
                }
                _ => (),
            }
        }

        // Walk over properties that are not walked otherwise, in parallel.
        // Authors are walked over above as part of the reference (above) but
        // doing the below allows for fallbacks for canonicalizing their affiliations
        // which may not be handled as part of that process.

        let authors = self
            .authors
            .iter_mut()
            .flatten()
            .map(|author| author.canonicalize());
        try_join_all(authors).await?;

        let references = self
            .references
            .iter_mut()
            .flatten()
            .map(|reference| reference.canonicalize());
        try_join_all(references).await?;

        Ok(())
    }
}

impl Canonicalize for Author {
    async fn canonicalize(&mut self) -> Result<()> {
        match self {
            Author::Person(person) => person.canonicalize().await,
            Author::Organization(org) => org.canonicalize().await,
            Author::AuthorRole(role) => role.canonicalize().await,
            _ => Ok(()),
        }
    }
}

impl Canonicalize for AuthorRole {
    async fn canonicalize(&mut self) -> Result<()> {
        match &mut self.author {
            AuthorRoleAuthor::Person(person) => person.canonicalize().await,
            AuthorRoleAuthor::Organization(org) => org.canonicalize().await,
            _ => Ok(()),
        }
    }
}

impl Canonicalize for Organization {
    async fn canonicalize(&mut self) -> Result<()> {
        if !is_ror(&self.ror) && !openalex::is_authorship_ror(&self.ror) {
            // Attempt to get ROR from OpenAlex, falling back to generating an
            // ROR from the hash of the organization
            let ror = match openalex::ror(&self.name).await? {
                Some(ror) => ror,
                None => cbor_hash::ror(self)?,
            };
            self.ror = Some(ror);
        }

        Ok(())
    }
}

impl Canonicalize for Person {
    async fn canonicalize(&mut self) -> Result<()> {
        if !is_orcid(&self.orcid) && !openalex::is_authorship_orcid(&self.orcid) {
            // Attempt to get ORCID from OpenAlex, falling back to generating an
            // ORCID from the hash of the person
            let orcid = match openalex::orcid(&self.family_names, &self.given_names).await? {
                Some(orcid) => orcid,
                None => cbor_hash::orcid(self)?,
            };
            self.orcid = Some(orcid);
        }

        // Walk over properties that are not walked otherwise, in parallel.

        let affiliations = self
            .affiliations
            .iter_mut()
            .flatten()
            .map(|org| org.canonicalize());
        try_join_all(affiliations).await?;

        Ok(())
    }
}

impl Canonicalize for Reference {
    async fn canonicalize(&mut self) -> Result<()> {
        // Canonicalize using OpenAlex. This will canonicalize the
        // DOI of the reference as well as the ORCIDs of the authors
        // and the RORs of their affiliations.
        openalex::reference(self).await?;

        // If the DOI is still missing then fallback to generating a DOI from the
        // hash of the reference
        if self.doi.is_none() {
            self.doi = Some(cbor_hash::doi_reference(self)?);
        }

        // Walk over properties that are not walked otherwise, in parallel.
        // This provides a fallback to canonicalize authors and their affiliations
        // if that was not done by OpenAlex canonicalization above

        let authors = self
            .authors
            .iter_mut()
            .flatten()
            .map(|author| author.canonicalize());
        try_join_all(authors).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_doi() {
        assert!(is_doi(&Some("10.7717/peerj.4375".into())));

        assert!(!is_doi(&Some("10.0000/openalex.W2741809807".into())));
        assert!(!is_doi(&Some("10.0000/stencila.aOoQvBTTtbA".into())));
    }

    #[test]
    fn test_is_orcid() {
        assert!(is_orcid(&Some("0000-0002-1825-0097".into())));

        assert!(!is_orcid(&Some("O000-0050-8313-8872".into())));
        assert!(!is_orcid(&Some("S327-4486-9489-6164".into())));
    }

    #[test]
    fn test_is_ror() {
        assert!(is_ror(&Some("02mhbdp94".into())));
        assert!(is_ror(&Some("03qxff017".into())));

        assert!(!is_ror(&Some("O4389424196".into())));
        assert!(!is_ror(&Some("Sddx6tq37".into())));
    }
}
