use std::hash::{Hash, Hasher};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};

use codec_cbor::r#trait::CborCodec;
use common::{eyre::Result, futures::future::try_join_all, seahash::SeaHasher};
use schema::{
    Article, Author, AuthorRole, AuthorRoleAuthor, Node, Organization, Person, Reference,
    VisitorAsync, WalkControl, WalkNode,
};

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
        if self.ror.is_some() {
            return Ok(());
        }

        // Fallback to generating a ROR from the hash of the organization
        self.ror = Some(hash_to_ror(self)?);

        Ok(())
    }
}

impl Canonicalize for Person {
    async fn canonicalize(&mut self) -> Result<()> {
        if self.orcid.is_some() {
            return Ok(());
        }

        // Fallback to generating an ORCID from the hash of the person
        self.orcid = Some(hash_to_orcid(self)?);

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
        self.doi = Some(hash_to_doi(self)?);

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

/// Hash a Stencila node
fn hash<T>(node: &T) -> Result<u64>
where
    T: CborCodec,
{
    let bytes = node.to_cbor()?;

    let mut hasher = SeaHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(hash)
}

/// Hash a Stencila node to a ROR-like string
///
/// Generates a string matching `^S[a-hj-km-np-tv-z0-9]{6}[0-9]{2}$`
/// by mapping bits of `n` into the 31‑char alphabet, then
/// appending `n % 100` as two decimal digits. See https://ror.readme.io/docs/identifier.
///
/// Uses a leading letter of S to indicate that this is a Stencila generated
/// pseudo-ROR.
fn hash_to_ror<T>(node: &T) -> Result<String>
where
    T: CborCodec,
{
    let int = hash(node)?;

    const CHARSET: &[u8] = b"abcdefghjkmnpqrstvwxyz0123456789";
    let base = CHARSET.len() as u64;
    let mut core = [0u8; 6];
    let mut x = int;
    for slot in core.iter_mut().rev() {
        *slot = CHARSET[(x % base) as usize];
        x /= base;
    }
    let middle = std::str::from_utf8(&core)?;

    Ok(format!("S{}{:02}", middle, int % 100))
}

/// Hash a Stencila node to a DOI-like string
///
/// Uses the example DOI prefix '10.0000' and 'stencila.' to indicate that
/// this is a pseudo-DOI whilst still being valid e.g.
///
/// 10.0000/stencila.QL-299Yo5YU
fn hash_to_doi<T>(node: &T) -> Result<String>
where
    T: CborCodec,
{
    let int = hash(node)?;
    let b64 = BASE64_URL_SAFE_NO_PAD.encode(int.to_be_bytes());
    Ok(format!("10.0000/stencila.{b64}"))
}

/// Hash a Stencila node to a ORCID-like string
///
/// Uses a leading letter S to indicate that this is a Stencila generated
/// pseudo-ORCID.
///
/// Note that the last digit of ORCIDs is a checksum so the generated ORCID
/// is likely to be invalid (which is a good thing in this case).
fn hash_to_orcid<T>(node: &T) -> Result<String>
where
    T: CborCodec,
{
    let int = hash(node)?;
    let digits = format!("{:015}", int % 1_000_000_000_000_000);
    Ok(format!(
        "S{}-{}-{}-{}",
        &digits[0..3],
        &digits[3..7],
        &digits[7..11],
        &digits[11..15],
    ))
}
