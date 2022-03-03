use archive_utils::extract_tar;
use octocrab::{models::User, params::repos::Commitish, Octocrab, OctocrabBuilder};
use provider::{
    async_trait::async_trait,
    codecs,
    eyre::Result,
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{
        CreativeWorkAuthors, CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Date,
        Node, Organization, Person, SoftwareSourceCode, ThingDescription,
    },
    tracing, Provider, ProviderParsing, ProviderTrait,
};
use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

pub struct GithubProvider;

impl GithubProvider {
    /// Create an API client
    fn client(token: Option<String>) -> Result<Octocrab> {
        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }
        let client = builder.build()?;
        Ok(client)
    }

    /// Extract the GitHub repository owner and name from a [`SoftwareSourceCode`] node (if any)
    fn owner_name(ssc: &SoftwareSourceCode) -> Option<(&str, &str)> {
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

    fn parse(string: &str) -> Vec<ProviderParsing> {
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
            .map(
                |(begin, end, org, name, version, content)| ProviderParsing {
                    begin,
                    end,
                    node: Node::SoftwareSourceCode(SoftwareSourceCode {
                        code_repository: Some(Box::new(format!(
                            "https://github.com/{}/{}",
                            org, name
                        ))),
                        publisher: Some(Box::new(CreativeWorkPublisher::Organization(
                            Organization {
                                name: Some(Box::new(org)),
                                ..Default::default()
                            },
                        ))),
                        name: Some(Box::new(name)),
                        version: version
                            .map(|version| Box::new(CreativeWorkVersion::String(version))),
                        content: content.map(|path| Box::new(CreativeWorkContent::String(path))),
                        ..Default::default()
                    }),
                },
            )
            .collect()
    }

    async fn enrich(node: Node) -> Result<Node> {
        let ssc = match &node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(node),
        };
        let (owner, name) = match GithubProvider::owner_name(ssc) {
            Some(owner_name) => owner_name,
            None => return Ok(node),
        };

        let client = GithubProvider::client(None)?;
        let repo = client.repos(owner, name);
        let repo = repo.get().await?;

        let description = repo
            .description
            .map(|desc| Box::new(ThingDescription::String(desc)));

        let keywords = repo.topics;

        let date_created = repo.created_at.map(|date| Box::new(Date::from(date)));
        let date_modified = repo.pushed_at.map(|date| Box::new(Date::from(date)));

        // TODO: Have a cache of users and fetch user data from URL if necessary
        let authors = match repo.contributors_url {
            Some(url) => {
                let contributors: Vec<User> =
                    client.get(url.path(), None::<&()>).await?;
                let authors = contributors
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
                Some(authors)
            }
            None => None,
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

    async fn import(node: &Node, destination: &Path, token: Option<String>) -> Result<bool> {
        let ssc = match node {
            Node::SoftwareSourceCode(ssc) => ssc,
            _ => return Ok(false),
        };
        let (owner, name) = match GithubProvider::owner_name(ssc) {
            Some(owner_name) => owner_name,
            None => return Ok(false),
        };
        let subpath = GithubProvider::path(ssc);
        let version = GithubProvider::version(ssc);

        let client = GithubProvider::client(token)?;
        let repo = client.repos(owner, name);

        // Fetch the contents at the path / version
        let mut builder = repo.get_content();
        if let Some(path) = subpath {
            builder = builder.path(path);
        }
        if let Some(version) = version {
            builder = builder.r#ref(version)
        }
        let content_items = builder.send().await?;
        let count = content_items.items.len();

        // No content (in directory)
        if count == 0 {
            tracing::warn!("No content at GitHub repository path/ref");
            return Ok(true);
        }

        // Content is a single file with content so write to destination
        if count == 1 {
            let content = &content_items.items[0];
            if content.r#type == "file" && content.content.is_some() {
                let name = PathBuf::from(&content.name);
                let string = content.decoded_content().unwrap_or_default();

                if let Some(dest_ext) = destination.extension() {
                    let dest_ext = dest_ext.to_string_lossy().to_string();
                    let source_ext = name.extension().map_or_else(
                        || content.name.to_string(),
                        |os_str| os_str.to_string_lossy().to_string(),
                    );
                    if source_ext == dest_ext {
                        // Destination format is same as content so just write it
                        let mut file = File::create(destination)?;
                        file.write_all(string.as_bytes())?;
                    } else {
                        // Destination has a different format so convert it first
                        codecs::str_to_path(&string, &source_ext, destination, None).await?;
                    }
                } else {
                    // Destination has no extension so treat it as a directory
                    // and write the file there
                    create_dir_all(destination)?;
                    let mut file = File::create(destination.join(name))?;
                    file.write_all(string.as_bytes())?;
                }
                return Ok(true);
            }
        }

        // Content is a directory so fetch the whole repo as a tarball and extract the directory
        // (getting the whole rpo as a tarball is more efficient than making lots of small requests
        // for each file's content; for most repos)
        let version = version.unwrap_or("HEAD").to_string();
        let commitish = Commitish::from(version);
        let response = repo.download_tarball(commitish).await?.error_for_status()?;

        let mut file = tempfile::NamedTempFile::new()?;
        let bytes = response.bytes().await?;
        io::copy(&mut bytes.as_ref(), &mut file)?;
        file.flush()?;
        let archive = file.path();

        create_dir_all(destination)?;
        extract_tar("gz", archive, destination, 1, subpath)?;

        Ok(true)
    }
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
