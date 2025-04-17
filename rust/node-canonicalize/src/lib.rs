use common::{eyre::Result, futures::future::try_join_all};
use schema::{
    Article, Author, AuthorRole, AuthorRoleAuthor, Node, Organization, Person, Reference,
    VisitorAsync, WalkControl, WalkNode,
};

mod cbor_hash;
mod open_alex;

/// Canonicalize a node
///
/// Sets the canonical id for `Article` and `Reference` nodes (DOI),
/// `Person` nodes (ORCID), and `Organization` nodes (ROR).
pub async fn canonicalize<T>(node: &mut T) -> Result<()>
where
    T: WalkNode,
{
    let mut walker = Walker::default();
    node.walk_async(&mut walker).await
}

#[derive(Default)]
struct Walker;

impl VisitorAsync for Walker {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        match node {
            Node::Article(node) => node.canonicalize().await?,

            // These node types are not normally canonicalized directly but are included
            // here primarily for tests
            Node::Organization(node) => node.canonicalize().await?,
            Node::Person(node) => node.canonicalize().await?,
            Node::Reference(node) => node.canonicalize().await?,

            _ => {}
        }
        Ok(WalkControl::Continue)
    }
}

trait Canonicalize {
    async fn canonicalize(&mut self) -> Result<()>;
}

impl Canonicalize for Article {
    async fn canonicalize(&mut self) -> Result<()> {
        if self.doi.is_none() {
            // Generate a DOI from the reference for the article
            let mut reference = Reference::from(&*self);
            reference.canonicalize().await?;
            self.doi = reference.doi;
        }

        // Walk over properties that are not walked otherwise, in parallel.

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
        if self.ror.is_none() {
            // Attempt to get ROR from OpenAlex, falling back to generating an
            // ROR from the hash of the organization
            let ror = match open_alex::ror(&self.name).await? {
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
        if self.orcid.is_none() {
            // Attempt to get ORCID from OpenAlex, falling back to generating an
            // ORCID from the hash of the person
            let orcid = match open_alex::orcid(&self.family_names, &self.given_names).await? {
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
            .map(|author| author.canonicalize());
        try_join_all(affiliations).await?;

        Ok(())
    }
}

impl Canonicalize for Reference {
    async fn canonicalize(&mut self) -> Result<()> {
        if self.doi.is_some() {
            return Ok(());
        }

        // Fallback to generating a DOI from the hash of the reference
        self.doi = Some(cbor_hash::doi(self)?);

        // Walk over properties that are not walked otherwise in parallel.

        let authors = self
            .authors
            .iter_mut()
            .flatten()
            .map(|author| author.canonicalize());
        try_join_all(authors).await?;

        Ok(())
    }
}
