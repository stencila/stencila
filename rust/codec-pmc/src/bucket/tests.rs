use std::{
    io,
    sync::{Arc, Mutex},
};

use stencila_codec::{
    eyre::{bail, eyre},
    stencila_format::Format,
    stencila_schema::Block,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::Mutex as AsyncMutex,
    task::JoinHandle,
};

use super::*;
use crate::decode::decode_pmcid_in;

const PMCID: &str = "PMC5697743";
const VERSIONS: &str = include_str!("../../tests/bucket/PMC5697743.versions.xml");
const OBJECTS: &str = include_str!("../../tests/bucket/PMC5697743.objects.xml");
const MANIFEST: &[u8] = include_bytes!("../../tests/bucket/article.json");
const XML: &[u8] = include_bytes!("../../tests/bucket/article.xml");
const IMAGE: &[u8] = include_bytes!("../../tests/bucket/emss-73944-f001.jpg");
const DATA: &[u8] = include_bytes!("../../tests/bucket/data.csv");
const EMPTY: &str = r#"<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><IsTruncated>false</IsTruncated></ListBucketResult>"#;

struct Reply {
    path: String,
    status: u16,
    body: Vec<u8>,
}

fn reply(path: &str, body: impl AsRef<[u8]>) -> Reply {
    Reply {
        path: path.into(),
        status: 200,
        body: body.as_ref().into(),
    }
}

/// A small scripted HTTP server verifies the actual requests, including the
/// query parameters and user agent. It has no dependency on external services.
///
/// Requests are matched to the first remaining reply with the same path, so
/// that concurrent object downloads may arrive in any order.
struct Server {
    base: String,
    replies: Arc<AsyncMutex<Vec<Reply>>>,
    errors: Arc<AsyncMutex<Vec<String>>>,
    task: JoinHandle<()>,
}

impl Server {
    async fn start(replies: Vec<Reply>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let base = format!("http://{}/", listener.local_addr()?);
        let replies = Arc::new(AsyncMutex::new(replies));
        let errors = Arc::new(AsyncMutex::new(Vec::new()));
        let task = tokio::spawn({
            let replies = replies.clone();
            let errors = errors.clone();
            async move {
                while let Ok((socket, _)) = listener.accept().await {
                    tokio::spawn(Self::respond(socket, replies.clone(), errors.clone()));
                }
            }
        });
        Ok(Self {
            base,
            replies,
            errors,
            task,
        })
    }

    async fn respond(
        mut socket: TcpStream,
        replies: Arc<AsyncMutex<Vec<Reply>>>,
        errors: Arc<AsyncMutex<Vec<String>>>,
    ) {
        // Read errors and early closes are from requests that the client
        // cancelled after another download failed.
        let mut request = Vec::new();
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let mut buffer = [0; 4096];
            match socket.read(&mut buffer).await {
                Ok(read) if read > 0 => request.extend_from_slice(&buffer[..read]),
                _ => return,
            }
        }
        let request = String::from_utf8_lossy(&request);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("GET "))
            .and_then(|line| line.strip_suffix(" HTTP/1.1"))
            .unwrap_or_default();
        let reply = {
            let mut replies = replies.lock().await;
            replies
                .iter()
                .position(|reply| reply.path == path)
                .map(|index| replies.remove(index))
        };
        let agent = format!("user-agent: {}", STENCILA_USER_AGENT.to_ascii_lowercase());
        let reply = match reply {
            Some(reply) if request.to_ascii_lowercase().contains(&agent) => reply,
            Some(_) => {
                errors.lock().await.push(format!("No user agent: {path}"));
                return;
            }
            None => {
                errors
                    .lock()
                    .await
                    .push(format!("Unexpected request: {path}"));
                Reply {
                    path: path.into(),
                    status: 404,
                    body: Vec::new(),
                }
            }
        };
        let header = format!(
            "HTTP/1.1 {} Response\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            reply.status,
            reply.body.len()
        );
        // Write errors are also from cancelled requests.
        let _ = socket.write_all(header.as_bytes()).await;
        let _ = socket.write_all(&reply.body).await;
        let _ = socket.shutdown().await;
    }

    /// Check that every request was expected and every reply was requested
    async fn finish(&self) -> Result<()> {
        self.finish_partial().await?;
        let replies = self.replies.lock().await;
        ensure!(
            replies.is_empty(),
            "Unrequested: {:?}",
            replies.iter().map(|reply| &reply.path).collect::<Vec<_>>()
        );
        Ok(())
    }

    /// Check that every request was expected. Used after a failed download,
    /// which cancels other requests that may not yet have been made.
    async fn finish_partial(&self) -> Result<()> {
        let errors = self.errors.lock().await;
        ensure!(errors.is_empty(), "Server errors: {errors:?}");
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// The object directory of the cache entry in `dir`
fn objects_dir(dir: &Path) -> Result<PathBuf> {
    let entry: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("PMC5697743.json"))?)?;
    let name = entry["directory"]
        .as_str()
        .ok_or_eyre("Missing cache directory")?;
    Ok(dir.join(name))
}

/// The names of files in `dir`, recursively
fn file_names(dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.extend(file_names(&entry.path())?);
        } else {
            names.push(entry.file_name().to_string_lossy().into());
        }
    }
    names.sort();
    Ok(names)
}

fn options() -> Option<DecodeOptions> {
    Some(DecodeOptions {
        embed_media: Some(false),
        embed_supplements: Some(false),
        reproducible: Some(false),
        ..Default::default()
    })
}

fn article_replies(manifest: &[u8], image: &[u8]) -> Vec<Reply> {
    // The recorded listing is extended with one tiny CSV supplement for this
    // reduced fixture. All other keys are real and must not be requested.
    let listing = OBJECTS.replace(
        "</ListBucketResult>",
        "<Contents><Key>PMC5697743.1/data.csv</Key></Contents></ListBucketResult>",
    );
    vec![
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", VERSIONS),
        reply("/?list-type=2&prefix=PMC5697743.1%2F", listing),
        reply("/PMC5697743.1/PMC5697743.1.json", manifest),
        reply("/PMC5697743.1/PMC5697743.1.xml", XML),
        reply("/PMC5697743.1/emss-73944-f001.jpg", image),
        reply("/PMC5697743.1/data.csv", DATA),
    ]
}

#[test]
fn highest_numeric_version_and_listing_validation() -> Result<()> {
    let listing = Listing::parse(
        r#"<ListBucketResult><IsTruncated>false</IsTruncated><CommonPrefixes><Prefix>PMC11193654.9/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>PMC11193654.319/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>PMC11193654.10/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>PMC111936540.999/</Prefix></CommonPrefixes></ListBucketResult>"#,
    )?;
    assert_eq!(
        listing.latest("PMC11193654")?,
        Some("PMC11193654.319/".into())
    );
    let recorded = Listing::parse(include_str!("../../tests/bucket/PMC11193654.objects.xml"))?;
    assert!(
        recorded
            .keys
            .contains(&"PMC11193654.319/PMC11193654.319.xml".into())
    );
    assert!(Listing::parse("<Error><Code>AccessDenied</Code></Error>").is_err());
    assert!(Listing::parse("<ListBucketResult/>").is_err());
    assert!(
        Listing::parse("<ListBucketResult><IsTruncated>true</IsTruncated></ListBucketResult>")
            .is_err()
    );
    assert!(Listing::parse(EMPTY)?.latest(PMCID)?.is_none());
    assert!(recorded.latest("PMC11193654").is_err());
    Ok(())
}

#[test]
fn recorded_manifests_and_safe_object_mapping() -> Result<()> {
    let manifest: Manifest =
        serde_json::from_str(include_str!("../../tests/bucket/PMC5697743.1.json"))?;
    assert!(manifest.is_manuscript && manifest.is_pmc_openaccess);
    assert_eq!(manifest.license_code, "TDM");
    assert_eq!(manifest.objects()?[0].filename, "PMC5697743.1.xml");
    let manuscript: Manifest =
        serde_json::from_str(include_str!("../../tests/bucket/PMC9524584.1.json"))?;
    assert!(manuscript.is_manuscript && !manuscript.is_pmc_openaccess);
    assert_eq!(manuscript.objects()?.len(), 1);
    let encoded = Object::parse(
        "s3://pmc-oa-opendata/PMC5697743.1/Figure%201%23a.jpg?md5=00000000000000000000000000000000",
        "PMC5697743.1",
    )?;
    assert_eq!(encoded.filename, "Figure 1#a.jpg");
    assert_eq!(
        Bucket::new(BUCKET_URL)?.object_url(&encoded.key).path(),
        "/PMC5697743.1/Figure%201%23a.jpg"
    );
    let plus = Object::parse(
        "s3://pmc-oa-opendata/PMC5697743.1/Table+S1.xlsx?md5=00000000000000000000000000000000",
        "PMC5697743.1",
    )?;
    assert_eq!(plus.filename, "Table+S1.xlsx");
    assert_eq!(
        Bucket::new("http://127.0.0.1:1/prefix")?
            .object_url(&plus.key)
            .path(),
        "/prefix/PMC5697743.1/Table%2BS1.xlsx"
    );
    for invalid in [
        "https://example.com/article.xml?md5=00000000000000000000000000000000",
        "s3://other-bucket/PMC5697743.1/article.xml?md5=00000000000000000000000000000000",
        "s3://pmc-oa-opendata/PMC5697743.2/article.xml?md5=00000000000000000000000000000000",
        "s3://pmc-oa-opendata/PMC5697743.1/nested/article.xml?md5=00000000000000000000000000000000",
        "s3://pmc-oa-opendata/PMC5697743.1/%2e%2e%2farticle.xml?md5=00000000000000000000000000000000",
        "s3://pmc-oa-opendata/PMC5697743.1/article.xml",
        "s3://pmc-oa-opendata/PMC5697743.1/article.xml?md5=bad",
    ] {
        assert!(
            Object::parse(invalid, "PMC5697743.1").is_err(),
            "Accepted {invalid}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn downloads_jats_media_supplements_metadata_and_uses_cache() -> Result<()> {
    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    let (node, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::PmcOa);
    let Node::Article(article) = &node else {
        bail!("Expected article")
    };
    assert!(article.options.licenses.as_ref().is_some_and(|licenses| {
        licenses.contains(&CreativeWorkVariantOrString::String("TDM".into()))
    }));
    let json = serde_json::to_value(&node)?;
    assert_eq!(json["pmc"]["is_retracted"], true);
    assert_eq!(json["pmc"]["is_manuscript"], true);
    assert_eq!(json["pmc"]["is_pmc_openaccess"], true);
    let Block::Figure(figure) = &article.content[1] else {
        bail!("Expected figure")
    };
    let Block::ImageObject(image) = &figure.content[0] else {
        bail!("Expected image")
    };
    assert_eq!(std::fs::read(&image.content_url)?, IMAGE);
    let Block::Supplement(supplement) = &article.content[2] else {
        bail!("Expected supplement")
    };
    assert_eq!(
        std::fs::read(
            supplement
                .target
                .as_deref()
                .ok_or_eyre("Missing supplement target")?
        )?,
        DATA
    );
    // Only the entry and the JATS, media, and supplement: no PDF or text.
    assert_eq!(
        file_names(dir.path())?,
        [
            "PMC5697743.1.xml",
            "PMC5697743.json",
            "data.csv",
            "emss-73944-f001.jpg"
        ]
    );
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 2);
    server.finish().await?;
    // The server has stopped; success now requires an entirely offline cache hit.
    let (cached, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::PmcOa);
    assert_eq!(serde_json::to_value(cached)?, json);

    // Exercise the defaults too: embed the image and decode the CSV supplement.
    let (embedded, _, _) = decode_bucket(
        &dir.path().join("PMC5697743.json"),
        Some(DecodeOptions {
            reproducible: Some(false),
            ..Default::default()
        }),
    )
    .await?;
    let embedded = serde_json::to_value(embedded)?;
    assert!(
        embedded["content"][1]["content"][0]["contentUrl"]
            .as_str()
            .is_some_and(|url| url.starts_with("data:image/"))
    );
    assert!(embedded["content"][2]["work"].is_object());
    Ok(())
}

#[tokio::test]
async fn checksum_mismatch_never_falls_back_or_commits_partial_cache() -> Result<()> {
    let server = Server::start(article_replies(MANIFEST, b"corrupt image")).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    let error = decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
        .await
        .err()
        .ok_or_eyre("Expected checksum error")?;
    assert!(error.to_string().contains("MD5 mismatch"));
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
    server.finish_partial().await?;
    Ok(())
}

#[tokio::test]
async fn cached_checksum_mismatch_is_an_error() -> Result<()> {
    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    bucket
        .download(PMCID, dir.path())
        .await?
        .ok_or_eyre("Missing article")?;
    server.finish().await?;
    std::fs::write(
        objects_dir(dir.path())?.join("PMC5697743.1.xml"),
        b"modified XML",
    )?;
    let error = decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
        .await
        .err()
        .ok_or_eyre("Expected cache error")?;
    assert!(error.to_string().contains("MD5 mismatch"));
    Ok(())
}

#[tokio::test]
async fn no_artifacts_references_outlive_temporary_downloads() -> Result<()> {
    // Own the exact side directories for the duration of the test, without
    // changing the process working directory or leaving files in the repo.
    let cwd = std::env::current_dir()?;
    let _media = tempfile::Builder::new()
        .prefix("PMC5697743.xml.media")
        .rand_bytes(0)
        .tempdir_in(&cwd)?;
    let _supplements = tempfile::Builder::new()
        .prefix("PMC5697743.xml.supplements")
        .rand_bytes(0)
        .tempdir_in(&cwd)?;
    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    let mut options = options().ok_or_eyre("Missing options")?;
    options.no_artifacts = Some(true);
    let (node, _, _) = decode_pmcid_in(
        PMCID,
        dir.path(),
        false,
        Some(options),
        &bucket,
        &server.base,
    )
    .await?;
    server.finish().await?;
    drop(dir);
    let Node::Article(article) = node else {
        bail!("Expected article")
    };
    let Block::Figure(figure) = &article.content[1] else {
        bail!("Expected figure")
    };
    let Block::ImageObject(image) = &figure.content[0] else {
        bail!("Expected image")
    };
    assert_eq!(std::fs::read(&image.content_url)?, IMAGE);
    let Block::Supplement(supplement) = &article.content[2] else {
        bail!("Expected supplement")
    };
    assert_eq!(
        std::fs::read(
            supplement
                .target
                .as_deref()
                .ok_or_eyre("Missing supplement target")?
        )?,
        DATA
    );
    Ok(())
}

#[tokio::test]
async fn ignored_cache_refresh_removes_stale_media_and_failed_refresh_preserves_cache() -> Result<()>
{
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST)?;
    manifest["is_pmc_openaccess"] = false.into();
    manifest["media_urls"] = serde_json::json!([]);
    let mut replies = article_replies(MANIFEST, IMAGE);
    replies.extend(article_replies(MANIFEST, b"corrupt"));
    let mut refreshed = article_replies(&serde_json::to_vec(&manifest)?, IMAGE);
    refreshed.truncate(4);
    replies.extend(refreshed);
    let server = Server::start(replies).await?;
    let dir = tempfile::tempdir()?;
    let bucket = Bucket::new(&server.base)?;
    decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert!(
        decode_pmcid_in(PMCID, dir.path(), true, options(), &bucket, &server.base)
            .await
            .is_err()
    );
    // A failed refresh leaves the previously verified cache intact.
    decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    let (_, info, _) =
        decode_pmcid_in(PMCID, dir.path(), true, options(), &bucket, &server.base).await?;
    assert_eq!(
        file_names(dir.path())?,
        ["PMC5697743.1.xml", "PMC5697743.json"]
    );
    assert!(serde_json::to_string(&info.losses)?.contains("unresolved: emss-73944-f001"));
    // Objects of the failed refresh may not have been requested.
    server.finish_partial().await?;
    Ok(())
}

#[tokio::test]
async fn manuscript_without_figures_decodes_with_losses() -> Result<()> {
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST)?;
    manifest["is_pmc_openaccess"] = false.into();
    manifest
        .as_object_mut()
        .ok_or_eyre("Expected JSON object")?
        .remove("media_urls");
    let mut replies = article_replies(&serde_json::to_vec(&manifest)?, IMAGE);
    replies.truncate(4);
    let server = Server::start(replies).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    let (node, info, _) =
        decode_pmcid_in(PMCID, dir.path(), false, None, &bucket, &server.base).await?;
    assert!(matches!(node, Node::Article(_)));
    let losses = serde_json::to_value(info.losses)?;
    assert!(
        losses
            .as_object()
            .ok_or_eyre("Expected losses")?
            .keys()
            .any(|key| key.contains("unresolved: emss-73944-f001"))
    );
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 2);
    server.finish().await?;
    Ok(())
}

#[tokio::test]
async fn paginates_versions_and_objects_and_encodes_tokens() -> Result<()> {
    let first = r#"<ListBucketResult><IsTruncated>true</IsTruncated><NextContinuationToken>a+/=&amp;b</NextContinuationToken><CommonPrefixes><Prefix>PMC5697743.9/</Prefix></CommonPrefixes></ListBucketResult>"#;
    let second = r#"<ListBucketResult><IsTruncated>false</IsTruncated><CommonPrefixes><Prefix>PMC5697743.319/</Prefix></CommonPrefixes><CommonPrefixes><Prefix>PMC5697743.10/</Prefix></CommonPrefixes></ListBucketResult>"#;
    let mut manifest: serde_json::Value = serde_json::from_slice(MANIFEST)?;
    manifest["version"] = 319.into();
    manifest["xml_url"] = format!(
        "s3://pmc-oa-opendata/PMC5697743.319/PMC5697743.319.xml?md5={:x}",
        Md5::digest(XML)
    )
    .into();
    manifest
        .as_object_mut()
        .ok_or_eyre("Expected JSON object")?
        .remove("media_urls");
    let server = Server::start(vec![
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", first),
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F&continuation-token=a%2B%2F%3D%26b", second),
        reply("/?list-type=2&prefix=PMC5697743.319%2F", r#"<ListBucketResult><IsTruncated>true</IsTruncated><NextContinuationToken>next</NextContinuationToken><Contents><Key>PMC5697743.319/PMC5697743.319.json</Key></Contents></ListBucketResult>"#),
        reply("/?list-type=2&prefix=PMC5697743.319%2F&continuation-token=next", r#"<ListBucketResult><IsTruncated>false</IsTruncated><Contents><Key>PMC5697743.319/PMC5697743.319.xml</Key></Contents></ListBucketResult>"#),
        reply("/PMC5697743.319/PMC5697743.319.json", serde_json::to_vec(&manifest)?),
        reply("/PMC5697743.319/PMC5697743.319.xml", XML),
    ]).await?;
    let dir = tempfile::tempdir()?;
    let bucket = Bucket::new(&server.base)?;
    assert!(bucket.download(PMCID, dir.path()).await?.is_some());
    assert!(objects_dir(dir.path())?.join("PMC5697743.319.xml").exists());
    server.finish().await?;
    Ok(())
}

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<u8>>>);

impl io::Write for Log {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("Poisoned log"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn only_empty_listing_falls_back_to_html_with_warning_and_caches() -> Result<()> {
    use tracing::instrument::WithSubscriber;
    let server = Server::start(vec![
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", EMPTY),
        reply(
            "/PMC5697743/",
            "<html><body><p>Fallback article</p></body></html>",
        ),
        // The cached page is used only while the bucket has no article.
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", EMPTY),
    ])
    .await?;
    let dir = tempfile::tempdir()?;
    let bucket = Bucket::new(&server.base)?;
    let log = Log::default();
    let writer = log.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .with_writer(move || writer.clone())
        .finish();
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
            .with_subscriber(subscriber)
            .await?;
    assert_eq!(format, Format::Html);
    let message = String::from_utf8(log.0.lock().map_err(|_| eyre!("Poisoned log"))?.clone())?;
    assert!(
        message.contains("WARN")
            && message.contains("empty listing")
            && message.contains("falling back to HTML")
    );
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::Html);
    server.finish().await?;
    Ok(())
}

#[tokio::test]
async fn bucket_errors_are_returned_without_html_fallback() -> Result<()> {
    for (status, body) in [(503, "Unavailable"), (200, "not XML")] {
        let mut first = reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", body);
        first.status = status;
        let server = Server::start(vec![first]).await?;
        let dir = tempfile::tempdir()?;
        let bucket = Bucket::new(&server.base)?;
        let error = decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
            .await
            .err()
            .ok_or_eyre("Expected bucket error")?;
        let reason = if status == 503 {
            "503"
        } else {
            "Invalid PMC bucket listing XML"
        };
        assert!(
            error.to_string().contains(reason),
            "Lost original bucket error: {error}"
        );
        assert!(!dir.path().join("PMC5697743.html").exists());
        server.finish().await?;
    }
    Ok(())
}

#[tokio::test]
async fn manifest_and_object_failures_do_not_fall_back() -> Result<()> {
    for failure in [
        "missing manifest",
        "bad JSON",
        "wrong identity",
        "object 404",
        "XML checksum",
    ] {
        let mut replies = article_replies(MANIFEST, IMAGE);
        match failure {
            "missing manifest" => {
                replies.truncate(2);
                replies[1].body = EMPTY.as_bytes().into();
            }
            "bad JSON" => {
                replies.truncate(3);
                replies[2].body = b"not JSON".into();
            }
            "wrong identity" => {
                replies.truncate(3);
                replies[2].body = String::from_utf8(MANIFEST.into())?
                    .replace("PMC5697743", "PMC0000000")
                    .into();
            }
            "object 404" => {
                replies[3].status = 404;
                replies[3].body.clear();
            }
            _ => {
                replies[3].body = b"corrupt JATS".into();
            }
        }
        let server = Server::start(replies).await?;
        let dir = tempfile::tempdir()?;
        let bucket = Bucket::new(&server.base)?;
        let error = decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
            .await
            .err()
            .ok_or_eyre("Expected bucket error")?;
        let reason = match failure {
            "missing manifest" => "no manifest",
            "bad JSON" => "Invalid PMC manifest JSON",
            "wrong identity" => "identity does not match",
            "object 404" => "404",
            _ => "MD5 mismatch",
        };
        assert!(
            error.to_string().contains(reason),
            "Lost original {failure} error: {error}"
        );
        assert!(!dir.path().join("PMC5697743.html").exists());
        assert!(!dir.path().join("PMC5697743.json").exists());
        server.finish_partial().await?;
    }
    Ok(())
}

#[tokio::test]
async fn cached_html_is_upgraded_to_bucket_unless_unavailable() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let html_path = dir.path().join("PMC5697743.html");
    std::fs::write(
        &html_path,
        "<html><body><p>Cached article</p></body></html>",
    )?;

    // Bucket errors, including being offline, fall back to the cached page.
    let mut unavailable = reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", "");
    unavailable.status = 503;
    let server = Server::start(vec![unavailable]).await?;
    let bucket = Bucket::new(&server.base)?;
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::Html);
    assert!(html_path.exists());
    server.finish().await?;

    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::PmcOa);
    assert!(!html_path.exists());
    server.finish().await?;
    Ok(())
}

#[tokio::test]
async fn html_refresh_retires_bucket_and_tar_caches() -> Result<()> {
    let mut replies = article_replies(MANIFEST, IMAGE);
    replies.extend([
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", EMPTY),
        reply(
            "/PMC5697743/",
            "<html><body><p>Fallback article</p></body></html>",
        ),
        reply("/?list-type=2&prefix=PMC5697743.&delimiter=%2F", EMPTY),
    ]);
    let server = Server::start(replies).await?;
    let bucket = Bucket::new(&server.base)?;
    let dir = tempfile::tempdir()?;
    decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    std::fs::write(dir.path().join("PMC5697743.tar.gz"), b"legacy package")?;

    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), true, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::Html);
    assert_eq!(file_names(dir.path())?, ["PMC5697743.html"]);

    // Neither the previous article nor the package take precedence.
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::Html);
    server.finish().await?;
    Ok(())
}

#[tokio::test]
async fn refresh_replaces_malformed_cache_entry() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("PMC5697743.json"), b"{\"truncated")?;
    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let error = decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base)
        .await
        .err()
        .ok_or_eyre("Expected cache entry error")?;
    assert!(error.to_string().contains("Invalid PMC cache entry"));

    decode_pmcid_in(PMCID, dir.path(), true, options(), &bucket, &server.base).await?;
    server.finish().await?;
    // The replacement is an entirely offline cache hit.
    let (_, _, format) =
        decode_pmcid_in(PMCID, dir.path(), false, options(), &bucket, &server.base).await?;
    assert_eq!(format, Format::PmcOa);
    Ok(())
}

#[tokio::test]
async fn cache_within_repository_has_no_local_git_provenance() -> Result<()> {
    // The crate directory is within the Stencila repository.
    let dir = tempfile::tempdir_in(std::env::current_dir()?)?;
    let server = Server::start(article_replies(MANIFEST, IMAGE)).await?;
    let bucket = Bucket::new(&server.base)?;
    let options = DecodeOptions {
        embed_media: Some(false),
        embed_supplements: Some(false),
        ..Default::default()
    };
    let (node, _, _) = decode_pmcid_in(
        PMCID,
        dir.path(),
        false,
        Some(options),
        &bucket,
        &server.base,
    )
    .await?;
    let Node::Article(article) = node else {
        bail!("Expected article")
    };
    assert_eq!(article.options.repository, None);
    assert_eq!(article.options.path, None);
    assert_eq!(article.options.commit, None);
    server.finish().await?;
    Ok(())
}
