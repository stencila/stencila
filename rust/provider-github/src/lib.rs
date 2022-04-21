use std::{
    env,
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
};

use archive_utils::extract_tar;
use hash_utils::bytes_hmac_sha256_hex;
use http_utils::{download_temp_with, headers, tempfile::NamedTempFile};
use octorust::types::{
    ContentFile, ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
    ReposGetContentResponseOneOf,
};
use provider::{
    async_trait::async_trait,
    codecs,
    eyre::{self, bail, eyre, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{
        CreativeWorkAuthors, CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Date,
        Node, Organization, Person, SoftwareSourceCode, ThingDescription,
    },
    tokens::token_for_provider,
    tokio::sync::mpsc,
    tracing, EnrichOptions, ImportOptions, ParseItem, Provider, ProviderTrait, SyncOptions,
};
use server_utils::{
    axum::{
        body::Bytes,
        http::{header::HeaderMap, StatusCode},
        response::Headers,
        routing, Router,
    },
    hostname, serde_json, serve_gracefully,
};

/// Port for the webhook server
///
/// This should not clash with any other port numbers for other providers.
/// Changes should be avoided as network configurations, such as firewall
/// rules, may assume this number.
const WEBHOOK_PORT: u16 = 10002;

/// The default name for the secret used to authenticate with the API
const SECRET_NAME: &str = "GITHUB_TOKEN";

/// A client for the Github REST API
///
/// This takes a just-in-time approach and gets the API token from the environment (if any)
/// and constucts an `octorust::Client` for each request (see the `api` method).
/// This is somewhat inefficient but allows the token to be added or changed without a restart.
#[derive(Clone)]
struct GithubClient {
    /// The name of the secret containing the access token
    secret_name: String,
}

impl GithubClient {
    /// Create a new Github API client
    fn new(secret_name: Option<String>) -> Self {
        let secret_name = secret_name.unwrap_or_else(|| SECRET_NAME.to_string());
        Self { secret_name }
    }

    /// Get an API token from the environment or Stencila API
    async fn token(&self) -> Result<Option<String>> {
        match env::var(&self.secret_name) {
            Ok(token) => Ok(Some(token)),
            Err(..) => token_for_provider("github").await,
        }
    }

    /// Get an `octorust` API client
    async fn api(&self) -> Result<octorust::Client> {
        let token = self.token().await?;
        let credentials = token.map(octorust::auth::Credentials::Token);
        Ok(octorust::Client::custom(
            octorust::DEFAULT_HOST,
            http_utils::USER_AGENT,
            credentials,
            http_utils::CLIENT.clone(),
        ))
    }

    /// Get additional headers required for a request
    ///
    /// Currenty only used for `download_temp`.
    async fn headers(&self) -> Result<Vec<(headers::HeaderName, String)>> {
        Ok(match self.token().await? {
            Some(token) => vec![(headers::AUTHORIZATION, ["Token ", &token].concat())],
            None => Vec::new(),
        })
    }

    /// Download the tarball for a repo
    ///
    /// octrorust's `download_tarball_archive` function returns a void so
    /// use `http_utils::download_temp_with` instead and "manually" add auth header
    async fn download_temp(&self, path: &str) -> Result<NamedTempFile> {
        let url = [octorust::DEFAULT_HOST, path].concat();
        let headers = self.headers().await?;
        download_temp_with(&url, None, &headers).await
    }
}

pub struct GithubProvider;

impl GithubProvider {
    /// Extract the GitHub repository owner and name from a [`SoftwareSourceCode`] node
    fn owner_repo(ssc: &SoftwareSourceCode) -> Result<(&str, &str)> {
        if let Some(repo) = &ssc.url {
            if let Some(repo) = repo.strip_prefix("https://github.com/") {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() >= 2 {
                    return Ok((parts[0], parts[1]));
                }
            }
        }
        bail!("Unable to resolve GitHub repo from `url` property")
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
            Regex::new(r"github:([a-zA-Z0-9\-]+)/([a-zA-Z0-9\-_]+)(?:/([^@\s]+))?(?:@([^\s]+))?")
                .expect("Unable to create regex")
        });

        // Regex targeting URL copied from the browser address bar
        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:https?://)?github\.com/([a-zA-Z0-9\-]+)/([a-zA-Z0-9\-_]+)/?(?:(?:tree|blob)/([^/\s]+)?/?([^\s]+))?(?:$|\s)")
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
                    url: Some(Box::new(format!("https://github.com/{}/{}", org, name))),
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

    fn recognize(node: &Node) -> bool {
        match node {
            Node::SoftwareSourceCode(ssc) => Self::owner_repo(ssc).is_ok(),
            _ => false,
        }
    }

    async fn enrich(node: Node, options: Option<EnrichOptions>) -> Result<Node> {
        let ssc = match &node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(node),
        };
        let (owner, repo) = match GithubProvider::owner_repo(ssc) {
            Ok(owner_repo) => owner_repo,
            Err(..) => return Ok(node),
        };

        let options = options.unwrap_or_default();

        let client = GithubClient::new(options.secret_name);

        let repo_details = client
            .api()
            .await?
            .repos()
            .get(owner, repo)
            .await
            .map_err(enhance_error)?;

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
            .api()
            .await?
            .repos()
            .list_all_contributors(owner, repo, "false")
            .await
            .map_err(enhance_error)?
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

    async fn import(node: &Node, dest: &Path, options: Option<ImportOptions>) -> Result<()> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => bail!("Unrecognized node type"),
        };
        let (owner, repo) = Self::owner_repo(ssc)?;
        let ref_ = Self::version(ssc);
        let path = Self::path(ssc);

        let options = options.unwrap_or_default();
        let client = GithubClient::new(options.secret_name.clone());

        let content = client
            .api()
            .await?
            .repos()
            .get_content(
                owner,
                repo,
                path.unwrap_or_default(),
                ref_.unwrap_or_default(),
            )
            .await
            .map_err(enhance_error)?;

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
                // Destination has no extension so treat it as a directory and write the file into it
                write_content_file(content_file, &dest.join(name))?;
            }
        } else if let ReposGetContentResponseOneOf::EntriesVector(entries) = content {
            if !entries.is_empty() {
                // Content is a directory so fetch the whole repo as a tarball and extract the directory
                // (getting the whole rpo as a tarball is more efficient than making lots of small requests
                // for each file's content - for most repos)
                let archive = client
                    .download_temp(&format!(
                        "/repos/{owner}/{repo}/tarball/{ref_}",
                        owner = owner,
                        repo = repo,
                        ref_ = ref_.unwrap_or_default()
                    ))
                    .await?;
                create_dir_all(dest)?;
                extract_tar("gz", archive.path(), dest, 1, path)?;
            }
        }

        Ok(())
    }

    async fn sync(
        node: &Node,
        local: &Path,
        _canceller: mpsc::Receiver<()>,
        options: Option<SyncOptions>,
    ) -> Result<()> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => bail!("Unrecognized node type"),
        };
        let (owner, repo) = Self::owner_repo(ssc)?;
        let version = Self::version(ssc);
        let path = Self::path(ssc);

        let options = options.unwrap_or_default();
        let client = GithubClient::new(options.secret_name);

        // Generate the unique id for the webhook
        let webhook_id = uuids::generate_num("wh", 36).to_string();

        // Create a URL for the webhook
        let webhook_host = match options.host {
            Some(host) => host,
            None => format!(
                "{hostname}:{port}",
                hostname = hostname().await,
                port = WEBHOOK_PORT
            ),
        };

        // Generate the secret key used for signing/validating event payloads
        let secret = key_utils::generate();

        // Create the webhook
        let config = ReposCreateWebhookRequestConfig {
            url: format!("https://{webhook_host}/{webhook_id}"),
            content_type: "json".to_string(),
            secret: secret.clone(),
            // Default values for properties that octorust does not provide defaults for
            insecure_ssl: None,
            token: "".to_string(),
            digest: "".to_string(),
        };
        let hook = client
            .api()
            .await?
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
            .map_err(enhance_error)?;
        tracing::info!("Created GitHub webhook `{}`", hook.url);

        // Listen for webhook events
        let response_headers = Headers(vec![(
            "Server",
            format!(
                "Stencila-GitHub-Provider/{} ({})",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS
            ),
        )]);
        let local_clone = local.to_path_buf();
        let client_clone = client.clone();
        let owner_clone = owner.to_string();
        let repo_clone = repo.to_string();
        let ref_ = version
            .map(|version| ["refs/heads/", version].concat())
            .unwrap_or_else(|| "refs/heads/main".to_string());
        let path_clone = path.map(|path| path.to_string()).unwrap_or_default();
        let router = Router::new().route(
            &format!("/{}", webhook_id),
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
                        &local_clone,
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
        serve_gracefully([0, 0, 0, 0], WEBHOOK_PORT, router).await?;

        // Delete the webhook
        match client
            .api()
            .await?
            .repos()
            .delete_webhook(owner, repo, hook.id)
            .await
        {
            Ok(..) => {
                tracing::info!("Deleted GitHub webhook `{}`", hook.id);
            }
            Err(error) => {
                tracing::warn!("While deleting GitHub webhook: {}", error);
            }
        }

        Ok(())
    }
}

/// Write a GitHub content file to disk
///
/// Weirdly, there are newlines in the Base64 encoding so this removes them first.
fn write_content_file(content_file: ContentFile, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?
    }
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
/// For debugging purposes this function both logs events and returns meaningful status codes
/// and messages for recording in the "Deliveries" log on GitHub.
/// See https://docs.github.com/en/developers/webhooks-and-events/webhooks/testing-webhooks
#[allow(clippy::too_many_arguments)]
async fn webhook_event(
    headers: HeaderMap,
    payload: Bytes,
    event: serde_json::Value,
    local: &Path,
    client: &GithubClient,
    secret: &str,
    owner: &str,
    repo: &str,
    ref_: &str,
    path: &str,
) -> Result<(StatusCode, String)> {
    // Reject events with a nonexistent or invalid HMAC signature
    let signature = match headers.get("X-Hub-Signature-256") {
        Some(value) => value.to_str()?,
        None => {
            let msg = "Rejected webhook event without signature";
            tracing::warn!("{}", msg);
            return Ok((StatusCode::BAD_REQUEST, msg.into()));
        }
    };
    if signature != ["sha256=", &bytes_hmac_sha256_hex(secret, payload.as_ref())?].concat() {
        let msg = "Rejected webhook event with invalid signature";
        tracing::warn!("{}", msg);
        return Ok((StatusCode::BAD_REQUEST, msg.into()));
    }

    // Reject events not associated with repo (should not happen but in case it does warn about it)
    let event_repo = event
        .pointer("/repository/full_name")
        .and_then(|action| action.as_str())
        .unwrap_or_default();
    let full_name = [owner, "/", repo].concat();
    if event_repo != full_name {
        let msg = format!(
            "Rejected webhook event for a different repo `{} != {}`",
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

                let local_path = match PathBuf::from(event_path).strip_prefix(path) {
                    // Only join stripped path if it has content. This avoids a trailing slash
                    // when the local path is a file
                    Ok(path) => match path == PathBuf::from("") {
                        true => local.to_path_buf(),
                        false => local.join(path),
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
                        local_path.display()
                    );
                    let content_file = client
                        .api()
                        .await?
                        .repos()
                        .get_content_file(owner, repo, event_path, event_ref)
                        .await
                        .map_err(enhance_error)?;
                    write_content_file(content_file, &local_path)?;
                } else {
                    // Remove the file, if it exists
                    if local_path.exists() {
                        remove_file(local_path)?;
                    } else {
                        tracing::warn!(
                            "Ignored webhook event to remove non-existent file `{}`",
                            local_path.display()
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

/// Convert an `anyhow::Error` to an `eyre::Report` and if the error is
/// a 404 provide the user with some hints as to what to do
///
/// See https://github.com/yaahc/eyre/issues/31 for potential improvements
fn enhance_error<E: std::fmt::Debug>(error: E) -> eyre::Report {
    let mut message = format!("{:?}", error);
    if message.contains("404 Not Found") {
        message = "Could not access the GitHub repository. Please check that it exists, that you have permission to access it, and a GitHub access token is available (you may need to connect GitHub to your Stencila account)".to_string();
    }
    eyre!(message)
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
            "github.com/owner/name/",
            "http://github.com/owner/name",
            "https://github.com/owner/name",
            "https://github.com/owner/name/",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                }
            );
        }

        // Version, no path
        for string in ["github:owner/name@version"] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/owner/name",
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
            "https://github.com/owner/name/tree/version/sub/folder",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/owner/name",
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
            "https://github.com/owner/name/blob/version/sub/folder/file.ext",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/owner/name",
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
        for string in ["github:owner/name/sub/folder/file.ext"] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                    "content": "sub/folder/file.ext"
                }
            );
        }

        // Capital letters and dashes in names or paths
        for string in [
            "github:Org-with-dashes/name-with-2Dashes/path-with/dashes-@branch-with-dashes-1",
            "github.com/Org-with-dashes/name-with-2Dashes/tree/branch-with-dashes-1/path-with/dashes-",
        ] {
            assert_json_is!(
                GithubProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "url": "https://github.com/Org-with-dashes/name-with-2Dashes",
                    "publisher": {
                        "type": "Organization",
                        "name": "Org-with-dashes"
                    },
                    "name": "name-with-2Dashes",
                    "version": "branch-with-dashes-1",
                    "content": "path-with/dashes-"
                }
            );
        }

        // Multiple items in a string
        let parse_items = GithubProvider::parse(
            "
            https://github.com/owner/name som word to be ignored
            and then another url github:owner/name
        ",
        );
        assert_eq!(parse_items.len(), 2);
        assert_json_is!(parse_items[0].node, {
            "type": "SoftwareSourceCode",
            "url": "https://github.com/owner/name",
            "publisher": {
                "type": "Organization",
                "name": "owner"
            },
            "name": "name"
        });
        assert_json_is!(parse_items[1].node, {
            "type": "SoftwareSourceCode",
            "url": "https://github.com/owner/name",
            "publisher": {
                "type": "Organization",
                "name": "owner"
            },
            "name": "name"
        });

        // GitHub URLs that should not get parsed because usually you just want to download the content
        // using the `HttpProvider`.
        for string in [
            "https://github.com/stencila/test/archive/refs/heads/master.zip",
            "https://github.com/stencila/test/releases/download/v0.0.0/archive.tar.gz",
        ] {
            let parse_items = GithubProvider::parse(string);
            assert!(parse_items.is_empty());
        }
    }
}
