//! Official C2PA trust-list cache.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration as StdDuration,
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use stencila_dirs::{DirType, get_app_dir};

use crate::{
    error::{Error, Result},
    media,
};

/// Official C2PA trust-list PEM URL.
pub const OFFICIAL_TRUST_LIST_URL: &str = "https://raw.githubusercontent.com/c2pa-org/conformance-public/refs/heads/main/trust-list/C2PA-TRUST-LIST.pem";

const TRUST_DIR: &str = "credentials/trust";
const TRUST_LIST_FILENAME: &str = "C2PA-TRUST-LIST.pem";
const TRUST_LIST_META_FILENAME: &str = "C2PA-TRUST-LIST.meta.json";
const TRUST_LIST_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;
const TRUST_LIST_HTTP_TIMEOUT_SECONDS: u64 = 10;

/// Status of the cached official C2PA trust list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustListStatus {
    pub url: &'static str,
    pub path: PathBuf,
    pub meta_path: PathBuf,
    pub present: bool,
    pub fresh: bool,
    pub fetched_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub sha256: Option<String>,
    pub ttl_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrustListMeta {
    url: String,
    fetched_at: DateTime<Utc>,
    sha256: String,
    ttl_seconds: i64,
}

/// Return the official trust-list PEM, refreshing the cache when missing or stale.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be created, the trust list
/// cannot be fetched, or cache files cannot be read or written.
pub async fn official_trust_anchors() -> Result<String> {
    if status()?.fresh {
        return read_cached_trust_anchors();
    }

    refresh_official_trust_list().await?;
    read_cached_trust_anchors()
}

/// Return official trust anchors without making verification depend on network access.
///
/// Uses a fresh cache when available, refreshes stale or missing cache entries
/// when possible, falls back to stale cached anchors when refresh fails, and
/// returns `None` when there is no usable cache.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be resolved or cached trust
/// anchors cannot be read.
pub async fn official_trust_anchors_best_effort() -> Result<Option<String>> {
    let status = status_best_effort()?;

    if status.fresh {
        return read_cached_trust_anchors().map(Some);
    }

    match refresh_official_trust_list().await {
        Ok(_) => read_cached_trust_anchors().map(Some),
        Err(error) => {
            tracing::debug!(
                ?error,
                "failed to refresh official C2PA trust list; continuing without refreshed anchors"
            );

            if status.present {
                return Ok(Some(fs::read_to_string(status.path)?));
            }

            Ok(None)
        }
    }
}

/// Refresh the official C2PA trust-list cache.
///
/// # Errors
///
/// Returns an error if the trust list cannot be fetched or written.
pub async fn refresh_official_trust_list() -> Result<TrustListStatus> {
    let pem = reqwest::Client::builder()
        .timeout(StdDuration::from_secs(TRUST_LIST_HTTP_TIMEOUT_SECONDS))
        .build()?
        .get(OFFICIAL_TRUST_LIST_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    if !pem.contains("-----BEGIN CERTIFICATE-----") {
        return Err(Error::other(
            "downloaded C2PA trust list is not a PEM bundle",
        ));
    }

    let dir = trust_dir(true)?;
    let path = trust_list_path_in(&dir);
    let meta_path = trust_list_meta_path_in(&dir);
    fs::write(&path, pem.as_bytes())?;

    let meta = TrustListMeta {
        url: OFFICIAL_TRUST_LIST_URL.to_string(),
        fetched_at: Utc::now(),
        sha256: media::sha256_bytes(pem.as_bytes()),
        ttl_seconds: TRUST_LIST_TTL_SECONDS,
    };
    fs::write(&meta_path, serde_json::to_vec_pretty(&meta)?)?;

    status()
}

/// Return current trust-list cache status.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be resolved or metadata
/// cannot be read.
pub fn status() -> Result<TrustListStatus> {
    let dir = trust_dir(false)?;
    status_in(&dir, true)
}

fn status_best_effort() -> Result<TrustListStatus> {
    let dir = trust_dir(false)?;
    status_in(&dir, false)
}

fn status_in(dir: &Path, meta_errors_fatal: bool) -> Result<TrustListStatus> {
    let path = trust_list_path_in(dir);
    let meta_path = trust_list_meta_path_in(dir);
    let meta = match read_meta(&meta_path) {
        Ok(meta) => meta,
        Err(error) if !meta_errors_fatal => {
            tracing::debug!(
                ?error,
                path = %meta_path.display(),
                "ignoring unreadable C2PA trust-list metadata"
            );
            None
        }
        Err(error) => return Err(error),
    };
    let present = path.exists();

    let actual_sha256 = if present {
        Some(media::sha256_bytes(&fs::read(&path)?))
    } else {
        None
    };

    let (fetched_at, expires_at, sha256, fresh) = if let Some(meta) = meta {
        let expires_at = meta.fetched_at + ChronoDuration::seconds(meta.ttl_seconds);
        let hash_matches = actual_sha256.as_deref() == Some(meta.sha256.as_str());
        let fresh = present && hash_matches && Utc::now() < expires_at;
        (
            Some(meta.fetched_at),
            Some(expires_at),
            actual_sha256.or(Some(meta.sha256)),
            fresh,
        )
    } else {
        (None, None, actual_sha256, false)
    };

    Ok(TrustListStatus {
        url: OFFICIAL_TRUST_LIST_URL,
        path,
        meta_path,
        present,
        fresh,
        fetched_at,
        expires_at,
        sha256,
        ttl_seconds: TRUST_LIST_TTL_SECONDS,
    })
}

fn read_cached_trust_anchors() -> Result<String> {
    let status = status()?;
    if !status.present {
        return Err(Error::InputNotFound(status.path));
    }
    Ok(fs::read_to_string(status.path)?)
}

fn read_meta(path: &Path) -> Result<Option<TrustListMeta>> {
    if !path.exists() {
        return Ok(None);
    }

    let meta = serde_json::from_slice(&fs::read(path)?)?;
    Ok(Some(meta))
}

fn trust_dir(ensure: bool) -> Result<PathBuf> {
    let cache_dir =
        get_app_dir(DirType::Cache, ensure).map_err(|err| Error::other(err.to_string()))?;
    let dir = cache_dir.join(TRUST_DIR);
    if ensure && !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

fn trust_list_path_in(dir: &Path) -> PathBuf {
    dir.join(TRUST_LIST_FILENAME)
}

fn trust_list_meta_path_in(dir: &Path) -> PathBuf {
    dir.join(TRUST_LIST_META_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_bytes_has_prefix() {
        assert!(media::sha256_bytes(b"abc").starts_with("sha256:"));
    }

    #[test]
    fn best_effort_status_ignores_malformed_metadata() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            trust_list_path_in(dir.path()),
            "-----BEGIN CERTIFICATE-----\n-----END CERTIFICATE-----",
        )?;
        fs::write(trust_list_meta_path_in(dir.path()), "{not json")?;

        let strict = status_in(dir.path(), true);
        assert!(matches!(strict, Err(Error::Json(_))));

        let lenient = status_in(dir.path(), false)?;
        assert!(lenient.present);
        assert!(!lenient.fresh);
        assert!(lenient.sha256.is_some());

        Ok(())
    }
}
