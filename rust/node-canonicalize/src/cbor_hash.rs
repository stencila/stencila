//! Functions for creating canonical ids from hashes of Stencila nodes serialized to CBOR

use std::hash::{Hash, Hasher};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};

use codec_cbor::r#trait::CborCodec;
use common::{eyre::Result, seahash::SeaHasher};

/// Hash a Stencila node to a ROR-like string
///
/// Generates a string matching `^S[a-hj-km-np-tv-z0-9]{6}[0-9]{2}$`
/// by mapping bits of `n` into the 31‑char alphabet, then
/// appending `n % 100` as two decimal digits. See https://ror.readme.io/docs/identifier.
///
/// Uses a leading letter of S to indicate that this is a Stencila generated
/// pseudo-ROR.
pub(super) fn ror<T>(node: &T) -> Result<String>
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
pub(super) fn doi<T>(node: &T) -> Result<String>
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
pub(super) fn orcid<T>(node: &T) -> Result<String>
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
