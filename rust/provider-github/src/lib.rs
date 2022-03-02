use octocrab::{models::User, OctocrabBuilder};
use provider::{
    eyre::Result,
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{
        CreativeWorkContent, CreativeWorkPublisher, CreativeWorkVersion, Node, Organization,
        SoftwareSourceCode,
    },
    Provider, ProviderParsing, ProviderTrait,
};

pub struct GithubProvider;

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

        // Regex targeting URL copies from th browser address bar
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

    #[ignore]
    #[tokio::test]
    async fn repos() -> Result<()> {
        let client = OctocrabBuilder::new().build()?;

        let repos = vec![("stencila", "stencila")];
        for (owner, name) in repos {
            let repo = client.repos(owner, name);

            let repo = repo.get().await?;

            let contributors: Vec<(String, String)> =
                if let Some(contributors_url) = repo.contributors_url {
                    let contributors: Vec<User> =
                        client.get(contributors_url.path(), None::<&()>).await?;
                    contributors
                        .into_iter()
                        .map(|user| (user.login, user.url.path().to_string()))
                        .collect()
                } else {
                    Vec::new()
                };

            // TODO: Have a cache of users and fetch user data from URL if necessary
            // TODO: Filter out users with [bot] in their login

            println!(
                "{},{},\"{}\",\"{}\",{},{},{},{},{},{},{},{},\"{}\"",
                owner,
                name,
                repo.description.unwrap_or_default(),
                repo.topics.unwrap_or_default().join(","),
                repo.license.map_or_else(|| "".to_string(), |l| l.name),
                repo.language
                    .map_or_else(|| "".to_string(), |v| v.as_str().unwrap().to_string()),
                repo.forks_count.unwrap_or(0),       // "Forks"
                repo.stargazers_count.unwrap_or(0),  // "Stars"
                repo.subscribers_count.unwrap_or(0), // "Watchers"
                repo.created_at.unwrap(),
                repo.pushed_at.unwrap(),
                repo.updated_at.unwrap(),
                contributors
                    .iter()
                    .map(|pair| pair.0.clone())
                    .collect::<Vec<String>>()
                    .join(",")
            );
        }

        Ok(())
    }
}
