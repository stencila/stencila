use octocrab::{models::User, OctocrabBuilder};
use provider::{eyre::Result, Provider, ProviderTrait};

/// A provider for GitHub
struct GitHubProvider {}

impl ProviderTrait for GitHubProvider {
    fn spec() -> Provider {
        Provider {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn repos() -> Result<()> {
        let client = OctocrabBuilder::new()
            .personal_token("...".to_string())
            .build()?;

        let repos = vec![("stencila", "stencila")];
        for (owner, name) in repos {
            let repo = client.repos(owner, name);

            let repo = repo.get().await?;

            let contributors: Vec<User> = client
                .get(repo.contributors_url.path(), None::<&()>)
                .await?;
            let contributors: Vec<(String, String)> = contributors
                .into_iter()
                .map(|user| (user.login, user.url.path().to_string()))
                .collect();
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
