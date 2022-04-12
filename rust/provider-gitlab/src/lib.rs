use std::{
    env,
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
};

use archive_utils::extract_tar;
use http_utils::{
    delete_with, download_temp_with, get_with, headers, post_with,
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    serde_json::json,
    tempfile::NamedTempFile,
};
use provider::{
    async_trait::async_trait,
    codecs,
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{
        CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Date, Node, Organization,
        SoftwareSourceCode, ThingDescription,
    },
    tokens::token_for_provider,
    tokio::sync::mpsc,
    tracing, EnrichOptions, ImportOptions, ParseItem, Provider, ProviderTrait, SyncOptions,
};
use server_utils::{
    axum::{
        http::{header::HeaderMap, StatusCode},
        response::Headers,
        routing, Json, Router,
    },
    hostname, serve_gracefully,
};

/// The base URL for API requests
const BASE_URL: &str = "https://gitlab.com/api/v4/";

/// The default name for the secret used to authenticate with the API
const SECRET_NAME: &str = "GITLAB_TOKEN";

/// Port for the webhook server
///
/// This should not clash with any other port numbers for other providers.
/// Changes should be avoided as network configurations, such as firewall
/// rules, may assume this number.
const WEBHOOK_PORT: u16 = 10003;

/// A client for the Gitlab REST API
///
/// Although there is an existing Rust client for the Gitlab REST API there
/// were several difficulties with using it:
///  - it is blocking and thus needs workarounds for use withing async functions
///  - it requires an access token event though the API allows some routes to be used without one
///  - it does not define many types for returned payloads (so they need to be coded up anyway)
///
/// Also, by using our existing HTTP client we benefit from caching.
///
/// This takes a just-in-time approach and gets the API token from the environment (if any) for
/// each request. This allows the token to be added or changed without a restart.
#[derive(Clone)]
struct GitlabClient {
    /// The name of the secret containing the access token
    secret_name: String,
}

impl GitlabClient {
    /// Create a new Gitlab API client
    fn new(secret_name: Option<String>) -> Self {
        let secret_name = secret_name.unwrap_or_else(|| SECRET_NAME.to_string());
        Self { secret_name }
    }

    /// Get an API token from the environment or Stencila API
    async fn token(&self) -> Result<Option<String>> {
        match env::var(&self.secret_name) {
            Ok(token) => Ok(Some(token)),
            Err(..) => token_for_provider("gitlab").await,
        }
    }

    /// Get additional headers required for a request
    async fn headers(&self) -> Result<Vec<(headers::HeaderName, String)>> {
        Ok(match self.token().await? {
            Some(token) => vec![(headers::AUTHORIZATION, ["Bearer ", &token].concat())],
            None => Vec::new(),
        })
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let headers = self.headers().await?;
        get_with(&[BASE_URL, path].concat(), &headers).await
    }

    async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: B) -> Result<T> {
        let headers = self.headers().await?;
        post_with(&[BASE_URL, path].concat(), body, &headers).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let headers = self.headers().await?;
        delete_with(&[BASE_URL, path].concat(), &headers).await
    }

    async fn download_temp(&self, path: &str) -> Result<NamedTempFile> {
        let headers = self.headers().await?;
        download_temp_with(&[BASE_URL, path].concat(), None, &headers).await
    }
}

pub struct GitlabProvider;

impl GitlabProvider {
    /// Extract the Gitlab org and project name from a [`SoftwareSourceCode`] node
    fn org_name(ssc: &SoftwareSourceCode) -> Result<(&str, &str)> {
        if let Some(repo) = &ssc.code_repository {
            if let Some(repo) = repo.strip_prefix("https://gitlab.com/") {
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() >= 2 {
                    return Ok((parts[0], parts[1]));
                }
            }
        }
        bail!("Unable to resolve Gitlab repository from `code_repository` property")
    }

    /// Extract the Gitlab project from a [`SoftwareSourceCode`] node as a URL encoded path
    ///
    /// See https://docs.gitlab.com/ee/api/index.html#namespaced-path-encoding
    fn project_id(ssc: &SoftwareSourceCode) -> Result<String> {
        Self::org_name(ssc).map(|(org, name)| format!("{}%2F{}", org, name))
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

    /// Extract the sub-path from a [`SoftwareSourceCode`] node as a URL encoded path
    fn path_id(ssc: &SoftwareSourceCode) -> Option<String> {
        Self::path(ssc).map(|path| urlencoding::encode(path).to_string())
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
impl ProviderTrait for GitlabProvider {
    fn spec() -> Provider {
        Provider::new("gitlab")
    }

    fn parse(string: &str) -> Vec<ParseItem> {
        // Regex targeting short identifiers e.g. gitlab:org/name
        static SHORT_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"gitlab:([a-zA-Z0-9\-]+)/([a-zA-Z0-9\-_]+)(?:/([^@\s]+))?(?:@([^\s]+))?")
                .expect("Unable to create regex")
        });

        // Regex targeting URL copied from the browser address bar
        // Note that compared to the equivalent Gitlab URLs, these have an additional `-/` before `tree` or `blob`
        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?:https?://)?gitlab\.com/([a-zA-Z0-9\-]+)/([a-zA-Z0-9\-_]+)/?(?:-/(?:tree|blob)/([^/\s]+)?/?([^\s]+))?(?:$|\s)")
                .expect("Unable to create regex")
        });

        SHORT_REGEX
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
                    code_repository: Some(Box::new(format!("https://gitlab.com/{}/{}", org, name))),
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
            Node::SoftwareSourceCode(ssc) => Self::project_id(ssc).is_ok(),
            _ => false,
        }
    }

    async fn enrich(node: Node, options: Option<EnrichOptions>) -> Result<Node> {
        let ssc = match &node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(node),
        };
        let project_id = match Self::project_id(ssc) {
            Ok(project_id) => project_id,
            Err(..) => return Ok(node),
        };

        let options = options.unwrap_or_default();

        let client = GitlabClient::new(options.secret_name);
        let project: Project = client.get(&format!("projects/{}", project_id)).await?;

        let description = match !project.description.is_empty() {
            true => Some(Box::new(ThingDescription::String(project.description))),
            false => None,
        };

        let date_created = Some(Box::new(Date::from(project.created_at)));

        let ssc = SoftwareSourceCode {
            description,
            date_created,
            ..ssc.clone()
        };

        Ok(Node::SoftwareSourceCode(ssc))
    }

    async fn import(node: &Node, dest: &Path, options: Option<ImportOptions>) -> Result<()> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => bail!("Unrecognized node type"),
        };
        let project_id = Self::project_id(ssc)?;

        let options = options.unwrap_or_default();
        let client = GitlabClient::new(options.secret_name);

        let ref_ = Self::version(ssc).unwrap_or("HEAD").to_string();
        let path = Self::path(ssc).unwrap_or_default();
        let path_id = Self::path_id(ssc).unwrap_or_default();

        // Attempt to get the path as a single file
        let repo_file = match client
            .get::<RepositoryFile>(&format!(
                "projects/{id}/repository/files/{path}?ref={ref_}",
                id = project_id,
                path = path_id,
                ref_ = ref_
            ))
            .await
        {
            Ok(repo_file) => Some(repo_file),
            Err(error) => {
                if !error.to_string().contains("404 Not Found") {
                    bail!("While fetching file from Gitlab: {}", error)
                } else {
                    // Not a file, so will attempt to extract as folder below
                    None
                }
            }
        };

        if let Some(repo_file) = repo_file {
            // Content is a single file with content so write to destination
            let name = PathBuf::from(&repo_file.file_name);
            if let Some(dest_ext) = dest.extension() {
                let dest_ext = dest_ext.to_string_lossy().to_string();
                let source_ext = name.extension().map_or_else(
                    || repo_file.file_name.to_string(),
                    |os_str| os_str.to_string_lossy().to_string(),
                );
                if source_ext == dest_ext {
                    // Destination format is same as content so just write it
                    repo_file.write(dest)?
                } else {
                    // Destination has a different format so convert it first
                    codecs::str_to_path(&repo_file.content()?, &source_ext, dest, None).await?;
                }
            } else {
                // Destination has no extension so treat it as a directory and write the file into it
                repo_file.write(&dest.join(&repo_file.file_name))?;
            }
        } else {
            // Content is a directory so fetch the whole repo as a tarball and extract the directory
            // (getting the whole rpo as a tarball is more efficient than making lots of small requests
            // for each file's content - for most repos)
            tracing::info!("Downloading repository tarball");
            let archive = client
                .download_temp(&format!(
                    "projects/{id}/repository/archive?sha={sha}&path={path}",
                    id = project_id,
                    sha = ref_,
                    path = path_id
                ))
                .await?;

            // Extract the part of the archive we want
            create_dir_all(dest)?;
            extract_tar(
                "gz",
                archive.path(),
                dest,
                1,
                if path.is_empty() { None } else { Some(path) },
            )?;
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
        let project_id = Self::project_id(ssc)?;
        let version = Self::version(ssc);
        let path = Self::path(ssc);

        let options = options.unwrap_or_default();
        let client = GitlabClient::new(options.secret_name);

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
        let webhook_url = format!("https://{webhook_host}/{webhook_id}");

        // Generate a token for validating event payloads
        let token = key_utils::generate().to_string();

        // Create the webhook
        let hook: Hook = client
            .post(
                &format!("projects/{id}/hooks", id = project_id),
                json!({
                    "url": webhook_url,
                    "token": token,
                    "push_events": true
                }),
            )
            .await?;
        tracing::info!("Created Gitlab webhook `{}`", hook.url);

        // Listen for webhook events
        let response_headers = Headers(vec![(
            "Server",
            format!(
                "Stencila-Gitlab-Provider/{} ({})",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS
            ),
        )]);
        let local_clone = local.to_path_buf();
        let client_clone = client.clone();
        let project_clone = project_id.clone();
        let ref_ = version
            .map(|version| ["refs/heads/", version].concat())
            .unwrap_or_else(|| "refs/heads/main".to_string());
        let path_clone = path.map(|path| path.to_string()).unwrap_or_default();
        let router = Router::new().route(
            &format!("/{}", webhook_id),
            routing::post(
                // Note that the order of extractors is important and some may not be able to be mixed.
                // See https://docs.rs/axum/latest/axum/extract/index.html#applying-multiple-extractors
                move |Json(event): Json<HookEvent>, request_headers: HeaderMap| async move {
                    match webhook_event(
                        request_headers,
                        event,
                        &local_clone,
                        &client_clone,
                        &token,
                        &project_clone,
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
            .delete(&format!(
                "projects/{project_id}/hooks/{hook_id}",
                project_id = project_id,
                hook_id = hook.id
            ))
            .await
        {
            Ok(..) => {
                tracing::info!("Deleted Gitlab webhook `{}`", hook.id);
            }
            Err(error) => {
                tracing::warn!("While deleting Gitlab webhook: {}", error);
            }
        }

        Ok(())
    }
}

/// A project
///
/// See https://docs.gitlab.com/ee/api/projects.html#get-single-project
#[derive(Debug, Deserialize)]
#[serde(crate = "http_utils::serde")]
struct Project {
    description: String,
    created_at: String,
}

/// A file in a repository
///
/// See https://docs.gitlab.com/ee/api/repository_files.html#get-file-from-repository
#[derive(Debug, Deserialize)]
#[serde(crate = "http_utils::serde")]
struct RepositoryFile {
    file_name: String,
    content: String,
}

impl RepositoryFile {
    /// Decode Base64 content to string
    fn content(&self) -> Result<String> {
        Ok(base64::decode(&self.content)
            .map(|slice| String::from_utf8_lossy(&slice).to_string())?)
    }

    /// Write content to disk
    fn write(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?
        }
        let mut file = File::create(path)?;
        file.write_all(&base64::decode(&self.content)?)?;
        Ok(())
    }
}

/// A webhook
///
/// See https://docs.gitlab.com/ee/api/projects.html#get-project-hook
#[derive(Debug, Deserialize)]
#[serde(crate = "http_utils::serde")]
struct Hook {
    id: u64,
    url: String,
}

/// A webhook push event
///
/// See https://docs.gitlab.com/ee/user/project/integrations/webhook_events.html#push-events
#[derive(Debug, Deserialize)]
#[serde(crate = "http_utils::serde")]
struct HookEvent {
    #[serde(rename = "ref")]
    ref_: String,
    commits: Vec<HookEventCommit>,
}

/// A commit within a webhook push event
#[derive(Debug, Deserialize)]
#[serde(crate = "http_utils::serde")]
struct HookEventCommit {
    added: Vec<String>,
    modified: Vec<String>,
    removed: Vec<String>,
}

/// Handle Gitlab webhook events
///
/// Validates payloads using secret token.
/// See https://docs.gitlab.com/ee/user/project/integrations/webhooks.html#validate-payloads-by-using-a-secret-token.
///
/// For debugging purposes this function both logs events and returns meaningful status codes
/// and messages for recording in the "Recent events" log on Gitlab.
/// See https://docs.gitlab.com/ee/user/project/integrations/webhooks.html#troubleshoot-webhooks
#[allow(clippy::too_many_arguments)]
async fn webhook_event(
    headers: HeaderMap,
    event: HookEvent,
    local: &Path,
    client: &GitlabClient,
    token: &str,
    project_id: &str,
    ref_: &str,
    path: &str,
) -> Result<(StatusCode, String)> {
    // Reject events with a nonexistent or invalid token
    match headers.get("X-Gitlab-Token") {
        Some(value) => {
            if value != token {
                let msg = "Rejected webhook event with an invalid token";
                tracing::warn!("{}", msg);
                return Ok((StatusCode::BAD_REQUEST, msg.into()));
            }
        }
        None => {
            let msg = "Rejected webhook event without token";
            tracing::warn!("{}", msg);
            return Ok((StatusCode::BAD_REQUEST, msg.into()));
        }
    };

    // Ignore events not associated with the ref being watched
    if event.ref_.is_empty() {
        // Not a push event
        let msg = "Ignoring non-push webhook event";
        tracing::trace!("{}", msg);
        return Ok((StatusCode::ACCEPTED, msg.into()));
    }
    if !(event.ref_ == ref_ || (event.ref_ == "refs/heads/master" && ref_ == "refs/heads/main")) {
        let msg = format!(
            "Ignoring webhook event for a different ref `{} != {}`",
            event.ref_, ref_
        );
        tracing::trace!("{}", msg);
        return Ok((StatusCode::ACCEPTED, msg));
    }

    // Iterate over the commits and synchronize each file
    for commit in event.commits {
        const ADDED: &str = "added";
        const MODIFIED: &str = "modified";
        const REMOVED: &str = "removed";
        for action in [ADDED, MODIFIED, REMOVED] {
            let paths = match action {
                ADDED => &commit.added,
                MODIFIED => &commit.modified,
                REMOVED => &commit.removed,
                _ => unreachable!(),
            };
            for event_path in paths {
                let local_path = match PathBuf::from(event_path).strip_prefix(path) {
                    // Only join stripped path if it has content. This avoids a trailing slash
                    // when the local path is a file
                    Ok(path) => match path == PathBuf::from("") {
                        true => local.to_path_buf(),
                        false => local.join(path),
                    },
                    Err(..) => {
                        tracing::info!(
                            "Ignored webhook event with excluded path: `{}` is not in `{}`",
                            event_path,
                            path
                        );
                        continue;
                    }
                };

                if action == ADDED || action == MODIFIED {
                    // Fetch the content of the file and write to disk
                    tracing::info!(
                        "Fetching content of `{}` to write to `{}`",
                        event_path,
                        local_path.display()
                    );
                    let repo_file = client
                        .get::<RepositoryFile>(&format!(
                            "projects/{id}/repository/files/{path}?ref={ref_}",
                            id = project_id,
                            path = urlencoding::encode(event_path),
                            ref_ = event.ref_
                        ))
                        .await?;
                    repo_file.write(&local_path)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_is;

    #[test]
    fn parse() {
        // No path or version
        for string in [
            "gitlab:owner/name",
            "gitlab.com/owner/name",
            "http://gitlab.com/owner/name",
            "https://gitlab.com/owner/name",
            "https://gitlab.com/owner/name/",
        ] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/owner/name",
                    "publisher": {
                        "type": "Organization",
                        "name": "owner"
                    },
                    "name": "name",
                }
            );
        }

        // Version, no path
        for string in ["gitlab:owner/name@version"] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/owner/name",
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
            "gitlab:owner/name/sub/folder@version",
            "https://gitlab.com/owner/name/-/tree/version/sub/folder",
        ] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/owner/name",
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
            "gitlab:owner/name/sub/folder/file.ext@version",
            "https://gitlab.com/owner/name/-/blob/version/sub/folder/file.ext",
        ] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/owner/name",
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

        // File path, no version (only for short identifier)
        for string in ["gitlab:owner/name/sub/folder/file.ext"] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/owner/name",
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
            "gitlab:Org-with-dashes/name-with-2Dashes/path-with/dashes-@branch-with-dashes-1",
            "gitlab.com/Org-with-dashes/name-with-2Dashes/-/tree/branch-with-dashes-1/path-with/dashes-",
            "https://gitlab.com/Org-with-dashes/name-with-2Dashes/-/tree/branch-with-dashes-1/path-with/dashes-",
        ] {
            assert_json_is!(
                GitlabProvider::parse(string)[0].node,
                {
                    "type": "SoftwareSourceCode",
                    "codeRepository": "https://gitlab.com/Org-with-dashes/name-with-2Dashes",
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
        let parse_items = GitlabProvider::parse(
            "
            https://gitlab.com/owner/name som word to be ignored
            and then another url gitlab:owner/name
        ",
        );
        assert_eq!(parse_items.len(), 2);
        assert_json_is!(parse_items[0].node, {
            "type": "SoftwareSourceCode",
            "codeRepository": "https://gitlab.com/owner/name",
            "publisher": {
                "type": "Organization",
                "name": "owner"
            },
            "name": "name"
        });
        assert_json_is!(parse_items[1].node, {
            "type": "SoftwareSourceCode",
            "codeRepository": "https://gitlab.com/owner/name",
            "publisher": {
                "type": "Organization",
                "name": "owner"
            },
            "name": "name"
        });

        // Gitlab URLs that should not get parsed because usually you just want to download the content
        // using the `HttpProvider`.
        for string in [
            "https://gitlab.com/stencila/test/archive/refs/heads/master.zip",
            "https://gitlab.com/stencila/test/releases/download/v0.0.0/archive.tar.gz",
        ] {
            let parse_items = GitlabProvider::parse(string);
            assert!(parse_items.is_empty());
        }
    }
}
