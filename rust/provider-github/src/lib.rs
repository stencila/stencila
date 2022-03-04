use std::{
    env,
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
};

use archive_utils::extract_tar;
use hash_utils::bytes_hmac_sha256_hex;
use http_utils::url;
use octorust::{
    auth::Credentials,
    types::{
        ContentFile, ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
        ReposGetContentResponseOneOf,
    },
    Client,
};
use provider::{
    async_trait::async_trait,
    codecs,
    eyre::{self, eyre, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{
        CreativeWorkAuthors, CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Date,
        Node, Organization, Person, SoftwareSourceCode, ThingDescription,
    },
    tracing, EnrichOptions, ImportOptions, ParseItem, Provider, ProviderTrait, WatchOptions,
};
use server_utils::{
    axum::{
        body::Bytes,
        http::{header::HeaderMap, StatusCode},
        response::Headers,
        routing, Router,
    },
    serde_json, serve_gracefully,
};
pub struct GithubProvider;

/// Default port for the webhook server
/// (the first 4 digits of "github" a1z26 encoded)
const WATCH_SERVER_PORT: u16 = 7920;

impl GithubProvider {
    /// Create an API client
    fn client(token: Option<String>) -> Client {
        Client::custom(
            octorust::DEFAULT_HOST,
            http_utils::USER_AGENT,
            token.map(Credentials::Token),
            http_utils::CLIENT.clone(),
        )
    }

    /// Extract the GitHub repository owner and name from a [`SoftwareSourceCode`] node (if any)
    fn owner_repo(ssc: &SoftwareSourceCode) -> Option<(&str, &str)> {
        if let Some(repo) = &ssc.code_repository {
            if let Some(repo) = repo.strip_prefix("https://github.com/") {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() >= 2 {
                    Some((parts[0], parts[1]))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract the sub-path from a [`SoftwareSourceCode`] node (if any)
    fn path(ssc: &SoftwareSourceCode) -> Option<&str> {
        ssc.content
            .as_ref()
            .and_then(|content| match content.as_ref() {
                CreativeWorkContent::String(path) => Some(path.as_str()),
                _ => None,
            })
    }

    /// Extract the version from a [`SoftwareSourceCode`] node (if any)
    fn version(ssc: &SoftwareSourceCode) -> Option<&str> {
        ssc.version
            .as_ref()
            .and_then(|version| match version.as_ref() {
                CreativeWorkVersion::String(version) => Some(version.as_str()),
                _ => None,
            })
    }
}

#[async_trait]
impl ProviderTrait for GithubProvider {
    fn spec() -> Provider {
        Provider::new("github")
    }

    fn parse(string: &str) -> Vec<ParseItem> {
        // Regex targeting short identifiers e.g. github:org/name
        static SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^github:(?://)?([a-z0-9\-]+)/([a-z0-9\-_]+)(?:/([^@]+))?(?:@(.+))?$")
                .expect("Unable to create regex")
        });

        // Regex targeting URL copied from the browser address bar
        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?:https?://)?github\.com/([a-z0-9\-]+)/([a-z0-9\-_]+)/?(?:(?:tree|blob))?/?([^/]+)?/?(.+)?$")
                .expect("Unable to create regex")
        });

        SIMPLE_REGEX
            .captures_iter(string)
            .into_iter()
            .map(|captures| {
                let capture = captures.get(0).unwrap();
                (
                    capture.start(),
                    capture.end(),
                    captures[1].to_string(),
                    captures[2].to_string(),
                    captures.get(4).map(|group| group.as_str().to_string()),
                    captures.get(3).map(|group| group.as_str().to_string()),
                )
            })
            .chain(URL_REGEX.captures_iter(string).into_iter().map(|captures| {
                let capture = captures.get(0).unwrap();
                (
                    capture.start(),
                    capture.end(),
                    captures[1].to_string(),
                    captures[2].to_string(),
                    captures.get(3).map(|group| group.as_str().to_string()),
                    captures.get(4).map(|group| group.as_str().to_string()),
                )
            }))
            .map(|(begin, end, org, name, version, content)| ParseItem {
                begin,
                end,
                node: Node::SoftwareSourceCode(SoftwareSourceCode {
                    code_repository: Some(Box::new(format!("https://github.com/{}/{}", org, name))),
                    publisher: Some(Box::new(CreativeWorkPublisher::Organization(
                        Organization {
                            name: Some(Box::new(org)),
                            ..Default::default()
                        },
                    ))),
                    name: Some(Box::new(name)),
                    version: version.map(|version| Box::new(CreativeWorkVersion::String(version))),
                    content: content.map(|path| Box::new(CreativeWorkContent::String(path))),
                    ..Default::default()
                }),
            })
            .collect()
    }

    async fn enrich(node: Node, options: Option<EnrichOptions>) -> Result<Node> {
        let ssc = match &node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(node),
        };
        let (owner, repo) = match GithubProvider::owner_repo(ssc) {
            Some(owner_repo) => owner_repo,
            None => return Ok(node),
        };

        let options = options.unwrap_or_default();

        let client = GithubProvider::client(options.token);

        let repo_details = client.repos().get(owner, repo).await.map_err(to_eyre)?;

        let description = match !repo_details.description.is_empty() {
            true => Some(Box::new(ThingDescription::String(repo_details.description))),
            false => None,
        };

        let keywords = match !repo_details.topics.is_empty() {
            true => Some(repo_details.topics),
            false => None,
        };

        let date_created = repo_details
            .created_at
            .map(|date| Box::new(Date::from(date)));

        let date_modified = repo_details
            .pushed_at
            .map(|date| Box::new(Date::from(date)));

        let authors = client
            .repos()
            .list_all_contributors(owner, repo, "false")
            .await
            .map_err(to_eyre)?
            .into_iter()
            .filter_map(|user| {
                if user.login.contains("[bot]") {
                    None
                } else {
                    Some(CreativeWorkAuthors::Person(Person {
                        name: Some(Box::new(user.login)),
                        ..Default::default()
                    }))
                }
            })
            .collect::<Vec<CreativeWorkAuthors>>();
        let authors = match !authors.is_empty() {
            true => Some(authors),
            false => None,
        };

        // TODO: Implement transforming these to ssc fields
        /*
        repo.license.map_or_else(|| "".to_string(), |l| l.name);
        repo.language
            .map_or_else(|| "".to_string(), |v| v.as_str().unwrap().to_string());
        repo.forks_count.unwrap_or(0);       // "Forks"
        repo.stargazers_count.unwrap_or(0);  // "Stars"
        repo.subscribers_count.unwrap_or(0); // "Watchers"
        */

        let ssc = SoftwareSourceCode {
            description,
            keywords,
            date_created,
            date_modified,
            authors,
            ..ssc.clone()
        };

        Ok(Node::SoftwareSourceCode(ssc))
    }

    async fn import(node: &Node, dest: &Path, options: Option<ImportOptions>) -> Result<bool> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(false),
        };
        let (owner, repo) = match GithubProvider::owner_repo(ssc) {
            Some(owner_repo) => owner_repo,
            None => return Ok(false),
        };

        let ref_ = GithubProvider::version(ssc);
        let path = GithubProvider::path(ssc);
        let options = options.unwrap_or_default();

        let client = GithubProvider::client(options.token.clone());

        let content = client
            .repos()
            .get_content(
                owner,
                repo,
                path.unwrap_or_default(),
                ref_.unwrap_or_default(),
            )
            .await
            .map_err(to_eyre)?;

        if let ReposGetContentResponseOneOf::ContentFile(content_file) = content {
            // Content is a single file with content so write to destination
            let name = PathBuf::from(&content_file.name);
            if let Some(dest_ext) = dest.extension() {
                let dest_ext = dest_ext.to_string_lossy().to_string();
                let source_ext = name.extension().map_or_else(
                    || content_file.name.to_string(),
                    |os_str| os_str.to_string_lossy().to_string(),
                );
                if source_ext == dest_ext {
                    // Destination format is same as content so just write it
                    write_content_file(content_file, dest)?
                } else {
                    // Destination has a different format so convert it first
                    codecs::str_to_path(&content_file.content, &source_ext, dest, None).await?;
                }
            } else {
                // Destination has no extension so treat it as a directory
                // and write the file there
                create_dir_all(dest)?;
                write_content_file(content_file, &dest.join(name))?;
            }
        } else if let ReposGetContentResponseOneOf::EntriesVector(entries) = content {
            if !entries.is_empty() {
                // Content is a directory so fetch the whole repo as a tarball and extract the directory
                // (getting the whole rpo as a tarball is more efficient than making lots of small requests
                // for each file's content -for most repos)

                // The octrorust's  `download_tarball_archive` function returns a void so
                // use `http_utils::download_temp_with` instead and "manually" add auth header
                let headers = match options.token {
                    Some(token) => vec![(
                        http_utils::headers::AUTHORIZATION,
                        format!("Token {}", token),
                    )],
                    None => Vec::new(),
                };
                let archive = http_utils::download_temp_with(
                    &format!(
                        "{host}/repos/{owner}/{repo}/tarball/{ref_}",
                        host = octorust::DEFAULT_HOST,
                        owner = owner,
                        repo = repo,
                        ref_ = ref_.unwrap_or_default()
                    ),
                    &headers,
                )
                .await?;

                // Extract the part of the archive we want
                create_dir_all(dest)?;
                extract_tar("gz", archive.path(), dest, 1, path)?;
            }
        }

        Ok(true)
    }

    async fn watch(node: &Node, dest: &Path, options: Option<WatchOptions>) -> Result<bool> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(false),
        };
        let (owner, repo) = match GithubProvider::owner_repo(ssc) {
            Some(owner_repo) => owner_repo,
            None => return Ok(false),
        };

        let version = GithubProvider::version(ssc);
        let path = GithubProvider::path(ssc);
        let options = options.unwrap_or_default();

        let client = GithubProvider::client(options.token);

        // Get a local URL
        let url = options.url.unwrap_or_default();
        let url = url::Url::parse(&url)?;

        // Generate the secret key used for signing/validating event payloads
        let secret = key_utils::generate();

        // Create the webhook
        let config = ReposCreateWebhookRequestConfig {
            url: Some(url),
            content_type: "json".to_string(),
            secret: secret.clone(),
            // Default values for properties that octorust does not provide defaults for
            insecure_ssl: None,
            token: "".to_string(),
            digest: "".to_string(),
        };
        let hook = client
            .repos()
            .create_webhook(
                owner,
                repo,
                &ReposCreateWebhookRequest {
                    name: "web".to_string(),
                    active: Some(true),
                    events: vec!["push".to_string()],
                    config: Some(config),
                },
            )
            .await
            .map_err(to_eyre)?;
        if let Some(url) = hook.url {
            tracing::info!("Created GitHub webhook `{}`", url);
        }

        // Listen for webhook events
        let response_headers = Headers(vec![(
            "Server",
            format!(
                "Stencila-GitHub-Provider/{} ({})",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS
            ),
        )]);
        let dest_clone = dest.to_path_buf();
        let client_clone = client.clone();
        let owner_clone = owner.to_string();
        let repo_clone = repo.to_string();
        let ref_ = version
            .map(|version| ["refs/heads/", version].concat())
            .unwrap_or_else(|| "refs/heads/main".to_string());
        let path_clone = path.map(|path| path.to_string()).unwrap_or_default();
        let router = Router::new().route(
            "/",
            routing::post(
                // Note that the order of extractors is important and some may not be able to be mixed
                // which is why we extract raw payload and then parse, rather than using the axum `Json` extractor.
                // See https://docs.rs/axum/latest/axum/extract/index.html#applying-multiple-extractors
                move |request_headers: HeaderMap, payload: Bytes| async move {
                    let event: serde_json::Value = match serde_json::from_slice(payload.as_ref()) {
                        Ok(event) => event,
                        Err(error) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                response_headers,
                                format!("Error parsing JSON: {}", error),
                            )
                        }
                    };
                    match webhook_event(
                        request_headers,
                        payload,
                        event,
                        &dest_clone,
                        &client_clone,
                        &secret,
                        &owner_clone,
                        &repo_clone,
                        &ref_,
                        &path_clone,
                    )
                    .await
                    {
                        Ok((status, msg)) => (status, response_headers, msg),
                        Err(error) => {
                            tracing::error!("While handling webhook event: {}", error);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                response_headers,
                                "Internal server error".into(),
                            )
                        }
                    }
                },
            ),
        );
        serve_gracefully([0, 0, 0, 0], WATCH_SERVER_PORT, router).await?;

        // Delete the webhook
        match client.repos().delete_webhook(owner, repo, hook.id).await {
            Ok(..) => {
                tracing::info!("Deleted GitHub webhook `{}`", hook.id);
            }
            Err(error) => {
                tracing::warn!("While deleting GitHub webhook: {}", error);
            }
        }

        Ok(true)
    }
}

/// Write a GitHub content file to disk
///
/// Weirdly, there are newlines in the Base64 encoding so this removes them first.
fn write_content_file(content_file: ContentFile, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    let content = content_file.content.replace("\n", "");
    file.write_all(&base64::decode(content)?)?;
    Ok(())
}

/// Handle GitHub webhook events
///
/// Validates payloads using HMAC signatures.
/// See https://docs.github.com/en/developers/webhooks-and-events/webhooks/securing-your-webhooks.
///
/// For debugging purposes this function logs events and returns meaningful status codes
/// and messages for recording in the "Deliveries" responses on GtHub.
/// See https://docs.github.com/en/developers/webhooks-and-events/webhooks/testing-webhooks
#[allow(clippy::too_many_arguments)]
async fn webhook_event(
    headers: HeaderMap,
    payload: Bytes,
    event: serde_json::Value,
    dest: &Path,
    client: &Client,
    secret: &str,
    owner: &str,
    repo: &str,
    ref_: &str,
    path: &str,
) -> Result<(StatusCode, String)> {
    // Ignore events with a nonexistent or invalid HMAC signature
    let signature = match headers.get("X-Hub-Signature-256") {
        Some(value) => value.to_str()?,
        None => {
            let msg = "Ignoring a webhook event without signature";
            tracing::warn!("{}", msg);
            return Ok((StatusCode::BAD_REQUEST, msg.into()));
        }
    };
    if signature != ["sha256=", &bytes_hmac_sha256_hex(secret, payload.as_ref())?].concat() {
        let msg = "Invalid webhook event signature";
        tracing::warn!("{}", msg);
        return Ok((StatusCode::BAD_REQUEST, msg.into()));
    }

    // Ignore events not associated with repo (should not happen but in case it does warn about it)
    let event_repo = event
        .pointer("/repository/full_name")
        .and_then(|action| action.as_str())
        .unwrap_or_default();
    let full_name = [owner, "/", repo].concat();
    if event_repo != full_name {
        let msg = format!(
            "Ignoring webhook event for a different repo `{} != {}`",
            event_repo, full_name
        );
        tracing::warn!("{}", msg);
        return Ok((StatusCode::BAD_REQUEST, msg));
    }

    // Ignore events not associated with the ref being watched
    let event_ref = event
        .get("ref")
        .and_then(|ref_| ref_.as_str())
        .unwrap_or_default();
    if event_ref.is_empty() {
        // Not a push event
        let msg = "Ignoring non-push webhook event";
        tracing::trace!("{}", msg);
        return Ok((StatusCode::ACCEPTED, msg.into()));
    }
    if !(event_ref == ref_ || (event_ref == "refs/heads/master" && ref_ == "refs/heads/main")) {
        let msg = format!(
            "Ignoring webhook event for a different ref `{} != {}`",
            event_ref, ref_
        );
        tracing::trace!("{}", msg);
        return Ok((StatusCode::ACCEPTED, msg));
    }

    // Iterate over the commits and synchronize each file
    let commits = event
        .get("commits")
        .and_then(|commits| commits.as_array())
        .into_iter()
        .flatten();
    for commit in commits {
        const ADDED: &str = "added";
        const MODIFIED: &str = "modified";
        const REMOVED: &str = "removed";
        for action in [ADDED, MODIFIED, REMOVED] {
            let paths = commit
                .get(action)
                .and_then(|paths| paths.as_array())
                .into_iter()
                .flatten();
            for event_path in paths {
                let event_path = event_path
                    .as_str()
                    .ok_or_else(|| eyre!("Expected path to be a string {}", path))?;

                let dest_path = match PathBuf::from(event_path).strip_prefix(path) {
                    // Only join stripped path if it has content. This avoids a trailing slash
                    // when the dest is a file
                    Ok(path) => match path == PathBuf::from("") {
                        true => dest.to_path_buf(),
                        false => dest.join(path),
                    },
                    Err(..) => {
                        tracing::trace!(
                            "Ignored webhook event with excluded path: `{}` is not in `{}`",
                            event_path,
                            path
                        );
                        continue;
                    }
                };

                if action == ADDED || action == MODIFIED {
                    // Fetch the content of the file and write to disk
                    tracing::trace!(
                        "Fetching content of `{}` to write to `{}`",
                        event_path,
                        dest_path.display()
                    );
                    let content_file = client
                        .repos()
                        .get_content_file(owner, repo, event_path, event_ref)
                        .await
                        .map_err(to_eyre)?;
                    if let Some(parent) = dest_path.parent() {
                        create_dir_all(parent)?
                    }
                    write_content_file(content_file, &dest_path)?;
                } else {
                    // Remove the file, if it exists
                    if dest_path.exists() {
                        remove_file(dest_path)?;
                    } else {
                        tracing::warn!(
                            "Ignored webhook event to remove non-existent file `{}`",
                            dest_path.display()
                        );
                    }
                }
            }
        }
    }

    let msg = "Webhook event handled";
    tracing::trace!("{}", msg);
    Ok((StatusCode::OK, msg.into()))
}

/// Convert an `anyhow::Error` to an `eyre::Report`
///
/// See https://github.com/yaahc/eyre/issues/31 for potential improvements
fn to_eyre<E: std::fmt::Debug>(error: E) -> eyre::Report {
    eyre!("{:?}", error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_is;

    #[test]
    fn parse() {
        // No path or version
        for string in [
            "github:owner/name",
            "github://owner/name",
            "http://github.com/owner/name",
            "https://github.com/owner/name",
            "https://github.com/owner/name/",
            "https://github.com/owner/name/tree",
            "https://github.com/owner/name/blob/",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                }
            );
        }

        // Version, no path
        for string in [
            "github:owner/name@version",
            "github://owner/name@version",
            "https://github.com/owner/name/tree/version",
            "https://github.com/owner/name/tree/version/",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                    "version": "version"
                }
            );
        }

        // Folder path and version
        for string in [
            "github:owner/name/sub/folder@version",
            "github://owner/name/sub/folder@version",
            "https://github.com/owner/name/tree/version/sub/folder",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                    "version": "version",
                    "content": "sub/folder"
                }
            );
        }

        // File path and version
        for string in [
            "github:owner/name/sub/folder/file.ext@version",
            "github://owner/name/sub/folder/file.ext@version",
            "https://github.com/owner/name/blob/version/sub/folder/file.ext",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                    "version": "version",
                    "content": "sub/folder/file.ext"
                }
            );
        }

        // File path, no version
        for string in [
            "github:owner/name/sub/folder/file.ext",
            "github://owner/name/sub/folder/file.ext",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                    "content": "sub/folder/file.ext"
                }
            );
        }
    }
}
