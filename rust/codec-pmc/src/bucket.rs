//! Anonymous HTTPS access to the PMC AWS Open Data bucket.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::{StreamExt, TryStreamExt, stream};
use md5::{Digest, Md5};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use reqwest::Client;
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use tempfile::Builder;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use url::Url;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result, WrapErr, ensure},
    stencila_schema::{CreativeWorkVariantOrString, Node, Primitive},
};
use stencila_codec_jats::JatsCodec;
use stencila_codec_utils::temp_file_for_atomic_write;
use stencila_version::STENCILA_USER_AGENT;

pub(super) const BUCKET_URL: &str = "https://pmc-oa-opendata.s3.amazonaws.com/";

/// Deadline for listings and manifests. Objects, which may be large
/// supplements, are bounded only by the client's connect and read timeouts.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Number of objects of an article downloaded concurrently
const CONCURRENT_DOWNLOADS: usize = 4;

/// Characters percent-encoded in object key segments: all but RFC 3986
/// unreserved characters. In particular, S3 reads a literal `+` as a space.
const KEY_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// One client is shared by all listings, object downloads, and the HTML fallback.
pub(super) struct Bucket {
    pub client: Client,
    base: Url,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Manifest {
    pmcid: String,
    version: u64,
    is_pmc_openaccess: bool,
    is_manuscript: bool,
    is_retracted: bool,
    license_code: String,
    xml_url: String,
    #[serde(default)]
    media_urls: Vec<String>,
}

impl Manifest {
    fn identity(&self) -> String {
        format!("{}.{}", self.pmcid, self.version)
    }

    fn objects(&self) -> Result<Vec<Object>> {
        let identity = self.identity();
        let xml = Object::parse(&self.xml_url, &identity)?;
        ensure!(
            xml.filename == format!("{identity}.xml"),
            "Unexpected PMC JATS key: {}",
            xml.key
        );
        let mut objects = vec![xml];
        let mut names = HashSet::from([objects[0].filename.clone()]);
        for url in &self.media_urls {
            let object = Object::parse(url, &identity)?;
            ensure!(
                names.insert(object.filename.clone()),
                "Duplicate PMC object: {}",
                object.key
            );
            // media_urls includes supplements, which the legacy tar route also
            // retained and optionally embedded. The article PDF and text are
            // separate manifest fields and are deliberately not requested.
            objects.push(object);
        }
        Ok(objects)
    }
}

struct Object {
    key: String,
    filename: String,
    md5: String,
}

impl Object {
    fn parse(value: &str, identity: &str) -> Result<Self> {
        let url = Url::parse(value).wrap_err("Invalid PMC object URL")?;
        ensure!(
            url.scheme() == "s3" && url.host_str() == Some("pmc-oa-opendata"),
            "Unexpected PMC object URL: {value}"
        );
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && url.fragment().is_none(),
            "Invalid PMC object URL: {value}"
        );
        let key = percent_decode_str(url.path()).decode_utf8()?;
        let key = key.trim_start_matches('/');
        let prefix = format!("{identity}/");
        let filename = key
            .strip_prefix(&prefix)
            .ok_or_eyre("PMC object is outside the article version")?;
        // The bucket uses flat directories. Decode before validating so encoded
        // separators cannot let a manifest write outside the cache.
        ensure!(
            is_plain_name(filename),
            "Invalid PMC object filename: {filename}"
        );
        let md5 = url
            .query_pairs()
            .find(|(key, _)| key == "md5")
            .map(|(_, value)| value.into_owned())
            .ok_or_eyre("PMC object has no MD5 checksum")?;
        ensure!(
            md5.len() == 32 && md5.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "Invalid PMC MD5 checksum for {key}"
        );
        Ok(Self {
            key: key.into(),
            filename: filename.into(),
            md5,
        })
    }
}

/// Whether a name is a single path component that stays within its directory
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\', '\0']) && !matches!(name, "." | "..")
}

/// The cache entry for an article: the manifest of the installed version and
/// the directory, beside the entry, holding its verified objects.
///
/// Each download is installed into a new directory and the entry is replaced
/// atomically, so a manuscript can only resolve objects of its own version and
/// an interrupted refresh never becomes a cache hit.
#[derive(Deserialize, Serialize)]
struct CacheEntry {
    directory: String,
    manifest: Manifest,
}

impl CacheEntry {
    async fn read(path: &Path) -> Result<Self> {
        let entry: Self = serde_json::from_slice(&tokio::fs::read(path).await?)
            .wrap_err_with(|| format!("Invalid PMC cache entry `{}`", path.display()))?;
        ensure!(
            is_plain_name(&entry.directory),
            "Invalid PMC cache directory: {}",
            entry.directory
        );
        Ok(entry)
    }

    /// The object directory of a possibly unreadable existing entry. An
    /// unreadable entry does not identify its directory, which must not
    /// prevent the entry being replaced or removed.
    async fn existing_directory(path: &Path) -> Option<PathBuf> {
        if !path.exists() {
            return None;
        }
        let parent = path.parent()?;
        match Self::read(path).await {
            Ok(entry) => Some(parent.join(entry.directory)),
            Err(error) => {
                tracing::warn!(
                    "{error:#}; any objects it referenced remain in `{}`",
                    parent.display()
                );
                None
            }
        }
    }
}

/// Remove an object directory that is no longer referenced by an entry
async fn remove_directory(directory: &Path) {
    if directory.exists()
        && let Err(error) = tokio::fs::remove_dir_all(directory).await
    {
        tracing::warn!(
            "Unable to remove superseded PMC objects `{}`: {error}",
            directory.display()
        );
    }
}

/// Remove a bucket cache entry and its objects, if any
pub(super) async fn remove(path: &Path) -> Result<()> {
    let directory = CacheEntry::existing_directory(path).await;
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    if let Some(directory) = directory {
        remove_directory(&directory).await;
    }
    Ok(())
}

/// The MD5 checksum of a file, without reading it entirely into memory
async fn file_md5(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path).await?;
    let mut digest = Md5::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[derive(Default)]
struct Listing {
    prefixes: Vec<String>,
    keys: Vec<String>,
    next: Option<String>,
}

impl Listing {
    fn parse(xml: &str) -> Result<Self> {
        let document = Document::parse(xml).wrap_err("Invalid PMC bucket listing XML")?;
        let root = document.root_element();
        ensure!(
            root.tag_name().name() == "ListBucketResult",
            "Unexpected PMC bucket listing response"
        );
        let mut listing = Self::default();
        let mut truncated = None;
        for child in root.children().filter(|node| node.is_element()) {
            match child.tag_name().name() {
                "CommonPrefixes" => {
                    let prefix = child
                        .children()
                        .find(|node| node.tag_name().name() == "Prefix")
                        .and_then(|node| node.text())
                        .ok_or_eyre("PMC listing prefix is missing")?;
                    listing.prefixes.push(prefix.into());
                }
                "Contents" => {
                    let key = child
                        .children()
                        .find(|node| node.tag_name().name() == "Key")
                        .and_then(|node| node.text())
                        .ok_or_eyre("PMC listing key is missing")?;
                    listing.keys.push(key.into());
                }
                "IsTruncated" => {
                    truncated = Some(child.text().unwrap_or_default().parse::<bool>()?)
                }
                "NextContinuationToken" => listing.next = child.text().map(String::from),
                _ => {}
            }
        }
        if truncated.ok_or_eyre("PMC listing has no IsTruncated flag")? {
            ensure!(
                listing.next.as_ref().is_some_and(|token| !token.is_empty()),
                "Truncated PMC listing has no continuation token"
            );
        } else {
            listing.next = None;
        }
        Ok(listing)
    }

    fn latest(&self, pmcid: &str) -> Result<Option<String>> {
        let prefix = format!("{pmcid}.");
        let latest = self
            .prefixes
            .iter()
            .filter_map(|value| {
                let version = value.strip_prefix(&prefix)?.strip_suffix('/')?;
                if version.is_empty() || !version.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                version.parse::<u64>().ok().map(|version| (version, value))
            })
            .max_by_key(|(version, _)| *version)
            .map(|(_, prefix)| prefix.clone());
        ensure!(
            latest.is_some() || (self.prefixes.is_empty() && self.keys.is_empty()),
            "PMC bucket listing has objects but no valid article version for {pmcid}"
        );
        Ok(latest)
    }
}

impl Bucket {
    pub fn new(base: &str) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent(STENCILA_USER_AGENT)
                .connect_timeout(Duration::from_secs(30))
                .read_timeout(Duration::from_secs(30))
                .build()?,
            base: Url::parse(base)?,
        })
    }

    fn object_url(&self, key: &str) -> Url {
        let key = key
            .split('/')
            .map(|segment| utf8_percent_encode(segment, KEY_SEGMENT).to_string())
            .collect::<Vec<_>>()
            .join("/");
        let mut url = self.base.clone();
        url.set_path(&format!("{}/{key}", self.base.path().trim_end_matches('/')));
        url
    }

    async fn listing(&self, prefix: &str, delimiter: Option<&str>) -> Result<Listing> {
        let mut result = Listing::default();
        let mut continuation: Option<String> = None;
        let mut seen = HashSet::new();
        loop {
            let mut url = self.base.clone();
            {
                let mut query = url.query_pairs_mut();
                query
                    .append_pair("list-type", "2")
                    .append_pair("prefix", prefix);
                if let Some(delimiter) = delimiter {
                    query.append_pair("delimiter", delimiter);
                }
                if let Some(token) = &continuation {
                    query.append_pair("continuation-token", token);
                }
            }
            let xml = self
                .client
                .get(url)
                .timeout(REQUEST_TIMEOUT)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            let page = Listing::parse(&xml)?;
            result.prefixes.extend(page.prefixes);
            result.keys.extend(page.keys);
            let Some(token) = page.next else { break };
            ensure!(
                seen.insert(token.clone()),
                "PMC bucket repeated a continuation token"
            );
            continuation = Some(token);
        }
        Ok(result)
    }

    /// Only an empty version listing means the article is absent. Every other
    /// failure must reach the caller rather than being masked by HTML scraping.
    pub async fn download(&self, pmcid: &str, directory: &Path) -> Result<Option<PathBuf>> {
        let versions = self.listing(&format!("{pmcid}."), Some("/")).await?;
        let Some(prefix) = versions.latest(pmcid)? else {
            return Ok(None);
        };
        let listing = self.listing(&prefix, None).await?;
        let identity = prefix.trim_end_matches('/');
        let manifest_key = format!("{prefix}{identity}.json");
        ensure!(
            listing.keys.contains(&manifest_key),
            "PMC bucket has no manifest for {identity}"
        );
        let json = self
            .client
            .get(self.object_url(&manifest_key))
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let manifest: Manifest =
            serde_json::from_slice(&json).wrap_err("Invalid PMC manifest JSON")?;
        ensure!(
            manifest.pmcid == pmcid && manifest.identity() == identity,
            "PMC manifest identity does not match {identity}"
        );
        let objects = manifest.objects()?;
        for object in &objects {
            ensure!(
                listing.keys.contains(&object.key),
                "PMC manifest object is missing from bucket listing: {}",
                object.key
            );
        }

        // Stage the entire article so failed downloads and checksum mismatches
        // cannot turn a partial article into a cache hit.
        tokio::fs::create_dir_all(directory).await?;
        let staging = Builder::new()
            .prefix(&format!("{identity}."))
            .tempdir_in(directory)?;
        // The futures are collected before being streamed, rather than created
        // by mapping the stream, because a closure over borrowed objects within
        // the stream is not general enough for it to be `Send` where awaited
        let downloads = objects
            .iter()
            .map(|object| self.download_object(object, staging.path().join(&object.filename)))
            .collect::<Vec<_>>();
        stream::iter(downloads)
            .buffer_unordered(CONCURRENT_DOWNLOADS)
            .try_collect::<()>()
            .await?;

        // Publish the staged directory by atomically replacing the entry, then
        // drop the superseded objects so they cannot accumulate.
        let entry_path = directory.join(format!("{pmcid}.json"));
        let previous = CacheEntry::existing_directory(&entry_path).await;
        let staged = staging.keep();
        let entry = CacheEntry {
            directory: staged
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_eyre("Invalid PMC staging directory")?
                .into(),
            manifest,
        };
        let temp_path = temp_file_for_atomic_write(&entry_path)?.into_temp_path();
        let persisted = async {
            tokio::fs::write(&temp_path, serde_json::to_vec(&entry)?).await?;
            temp_path.persist(&entry_path)?;
            Result::<()>::Ok(())
        }
        .await;
        if let Err(error) = persisted {
            remove_directory(&staged).await;
            return Err(error);
        }
        if let Some(previous) = previous {
            remove_directory(&previous).await;
        }
        Ok(Some(entry_path))
    }

    async fn download_object(&self, object: &Object, path: PathBuf) -> Result<()> {
        let response = self
            .client
            .get(self.object_url(&object.key))
            .send()
            .await?
            .error_for_status()?;
        let mut stream = response.bytes_stream();
        let mut file = File::create(path).await?;
        let mut digest = Md5::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            digest.update(&chunk);
            file.write_all(&chunk).await?;
        }
        ensure!(
            format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&object.md5),
            "PMC MD5 mismatch for {}",
            object.key
        );
        file.sync_all().await?;
        Ok(())
    }
}

/// Decode a completed bucket cache using the existing JATS loose-file and
/// media handling. JATS records unresolved figures as losses, including for
/// manuscripts whose manifest has no media_urls.
pub(super) async fn decode_bucket(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    let (node, info, _) = decode_bucket_entry(path, options).await?;
    Ok((node, None, info))
}

/// Decode a completed bucket cache, also returning the path of the JATS file
/// that any media references are relative to
pub(super) async fn decode_bucket_entry(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo, PathBuf)> {
    let entry = CacheEntry::read(path).await?;
    let directory = path
        .parent()
        .ok_or_eyre("PMC cache entry path has no parent")?
        .join(&entry.directory);
    let manifest = entry.manifest;
    let objects = manifest.objects()?;
    // Verify cache hits too: a lost or modified media object must not silently
    // degrade a previously complete article.
    for object in &objects {
        let md5 = file_md5(&directory.join(&object.filename))
            .await
            .wrap_err_with(|| format!("Unable to read cached PMC object {}", object.key))?;
        ensure!(
            md5.eq_ignore_ascii_case(&object.md5),
            "PMC MD5 mismatch for cached {}",
            object.key
        );
    }
    let jats_path = directory.join(&objects[0].filename);
    let options = options.unwrap_or_default();
    let embed_media = options.embed_media.unwrap_or(true);
    let embed_supplements = options.embed_supplements.unwrap_or(true);
    let mut jats_options = options.clone();
    jats_options.embed_media = Some(embed_media);
    jats_options.embed_supplements = Some(embed_supplements);
    // The cache may be within a local repository, whose Git details are not
    // the provenance of a downloaded article.
    jats_options.reproducible = Some(false);
    let (mut node, _, info) = JatsCodec.from_path(&jats_path, Some(jats_options)).await?;

    if let Node::Article(article) = &mut node {
        let licenses = article.options.licenses.get_or_insert_default();
        let license = CreativeWorkVariantOrString::String(manifest.license_code);
        if !licenses.contains(&license) {
            licenses.push(license);
        }
        let metadata = [
            (
                "is_pmc_openaccess",
                Primitive::Boolean(manifest.is_pmc_openaccess),
            ),
            ("is_manuscript", Primitive::Boolean(manifest.is_manuscript)),
            ("is_retracted", Primitive::Boolean(manifest.is_retracted)),
        ]
        .into();
        article
            .options
            .extra
            .get_or_insert_default()
            .insert("pmc".into(), Primitive::Object(metadata));
    }

    Ok((node, info, jats_path))
}

#[cfg(test)]
mod tests;
