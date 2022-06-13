use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
};

use octorust::types::{ContentFile, ReposGetContentResponseOneOf};

use archive_utils::extract_tar;
use provider::{
    codecs,
    common::{
        async_trait::async_trait,
        base64,
        eyre::{bail, eyre, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        serde_json,
        tempfile::NamedTempFile,
        tracing,
    },
    http_utils::{
        self, download_temp_with,
        http::{
            header::{self, HeaderName},
            Request, Response, StatusCode,
        },
        response,
    },
    resolve_token,
    stencila_schema::{
        CreativeWorkAuthors, CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Date,
        Node, Organization, Person, SoftwareSourceCode, ThingDescription,
    },
    EnrichOptions, ImportOptions, ParseItem, Provider, ProviderTrait, SyncOptions,
};

/// The default name for the token used to authenticate with the API
const TOKEN_NAME: &str = "GITHUB_TOKEN";

/// A client for the Github REST API
///
/// This takes a just-in-time approach and gets the API token from the environment (if any)
/// and constructs an `octorust::Client` for each request (see the `api` method).
/// This is somewhat inefficient but allows the token to be added or changed without a restart.
#[derive(Clone)]
struct GithubClient {
    /// A GitHub API token
    token: Option<String>,
}

impl GithubClient {
    /// Create a new Github API client
    fn new(token: Option<String>) -> Self {
        let token = resolve_token(token.as_deref().unwrap_or(TOKEN_NAME));
        Self { token }
    }

    /// Get an `octorust` API client
    async fn api(&self) -> Result<octorust::Client> {
        let credentials = self
            .token
            .as_ref()
            .map(|token| octorust::auth::Credentials::Token(token.to_string()));
        Ok(octorust::Client::custom(
            octorust::DEFAULT_HOST,
            http_utils::USER_AGENT,
            credentials,
            http_utils::CLIENT.clone(),
        ))
    }

    /// Get additional headers required for a request
    ///
    /// Currently only used for `download_temp`.
    fn headers(&self) -> Vec<(HeaderName, String)> {
        match &self.token {
            Some(token) => vec![(header::AUTHORIZATION, ["Token ", token].concat())],
            None => Vec::new(),
        }
    }

    /// Download the tarball for a repo
    ///
    /// octrorust's `download_tarball_archive` function returns a void so
    /// use `http_utils::download_temp_with` instead and "manually" add auth header
    async fn download_temp(&self, path: &str) -> Result<NamedTempFile> {
        let url = [octorust::DEFAULT_HOST, path].concat();
        let headers = self.headers();
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

        let client = GithubClient::new(options.token);

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
        let client = GithubClient::new(options.token.clone());

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

    #[tracing::instrument(skip(request))]
    async fn sync(
        node: &Node,
        dest: &Path,
        request: &Request<serde_json::Value>,
        options: Option<SyncOptions>,
    ) -> Result<Response<String>> {
        tracing::trace!("Received a GitHub sync event");

        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => bail!("Unrecognized node type"),
        };
        let (owner, repo) = Self::owner_repo(ssc)?;
        let version = Self::version(ssc);
        let sub_path = Self::path(ssc).unwrap_or_default();

        let options = options.unwrap_or_default();
        let client = GithubClient::new(options.token);

        let event = request.body();

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
            return Ok(response(StatusCode::BAD_REQUEST, &msg));
        }

        // Ignore events not associated with the ref being watched
        let desired_ref = version
            .map(|version| ["refs/heads/", version].concat())
            .unwrap_or_else(|| "refs/heads/main".to_string());
        let event_ref = event
            .get("ref")
            .and_then(|ref_| ref_.as_str())
            .unwrap_or_default();
        if event_ref.is_empty() {
            // Not a push event
            let msg = "Ignoring non-push webhook event";
            tracing::trace!("{}", msg);
            return Ok(response(StatusCode::ACCEPTED, msg));
        }
        if !(event_ref == desired_ref
            || (event_ref == "refs/heads/master" && desired_ref == "refs/heads/main"))
        {
            let msg = format!(
                "Ignoring webhook event for a different ref `{} != {}`",
                event_ref, desired_ref
            );
            tracing::trace!("{}", msg);
            return Ok(response(StatusCode::ACCEPTED, &msg));
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
                        .ok_or_else(|| eyre!("Expected path to be a string {}", event_path))?;

                    let dest_path = match PathBuf::from(event_path).strip_prefix(sub_path) {
                        // Only join stripped path if it has content. This avoids a trailing slash
                        // when the local path is a file
                        Ok(path) => match path == PathBuf::from("") {
                            true => dest.to_path_buf(),
                            false => dest.join(path),
                        },
                        Err(..) => {
                            tracing::trace!(
                                "Ignored webhook event with excluded path: `{}` is not in `{}`",
                                event_path,
                                sub_path
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
                            .api()
                            .await?
                            .repos()
                            .get_content_file(owner, repo, event_path, event_ref)
                            .await
                            .map_err(enhance_error)?;
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

        Ok(response(StatusCode::OK, "OK"))
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
    let content = content_file.content.replace('\n', "");
    file.write_all(&base64::decode(content)?)?;
    Ok(())
}

/// Convert an `anyhow::Error` to an `eyre::Report` and if the error is
/// a 404 provide the user with some hints as to what to do
///
/// See https://github.com/yaahc/eyre/issues/31 for potential improvements
fn enhance_error<E: std::fmt::Debug>(error: E) -> provider::common::eyre::Report {
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
