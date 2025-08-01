use std::{
    env,
    fs::{read_to_string, write},
    path::PathBuf,
};

use common_dev::pretty_assertions::assert_eq;
use kernel_docsql::DocsQLKernelInstance;
use kernel_jinja::kernel::{
    KernelInstance,
    common::{
        eyre::{Result, bail},
        glob::glob,
        itertools::Itertools,
        reqwest::{Client, header::USER_AGENT},
        tokio::{
            self,
            sync::{mpsc, watch},
        },
    },
    schema::{CodeChunk, Node, Null},
};
use version::STENCILA_USER_AGENT;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn golden() -> Result<()> {
    let (.., receiver) = watch::channel(Node::Null(Null));
    let (sender, ..) = mpsc::channel(1);
    let mut kernel = DocsQLKernelInstance::new(None, Some((receiver, sender)))?;

    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .canonicalize()?
        .to_string_lossy()
        .to_string();

    let test_paths: Vec<PathBuf> = if let Ok(glob_pattern) = env::var("GLOB") {
        let pattern_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .canonicalize()?
            .to_string_lossy()
            .to_string()
            + "/"
            + &glob_pattern;
        glob(&pattern_path)?.flatten().collect()
    } else {
        glob(&(pattern.clone() + "/*.cypher"))?
            .flatten()
            .chain(glob(&(pattern.clone() + "/*.openalex"))?.flatten())
            .chain(glob(&(pattern + "/*.github"))?.flatten())
            .collect()
    };

    let update = env::var("UPDATE_GOLDEN").is_ok();

    for path in test_paths {
        let contents = read_to_string(&path)?;

        let Some(filename) = path.file_name() else {
            continue;
        };
        let filename = filename.to_string_lossy();

        let mut tests = contents.split("\n\n").map(String::from).collect_vec();
        for test in tests.iter_mut() {
            let mut parts = test.split("---");

            let docsql = parts.next().unwrap_or_default().trim();
            if docsql.is_empty() {
                continue;
            }

            let (outputs, messages) = kernel.execute(&format!("{docsql}.explain()")).await?;

            let actual = if let Some(message) = messages.first() {
                message.message.clone()
            } else {
                match outputs.first() {
                    Some(Node::CodeChunk(CodeChunk { code, .. })) => code
                        .lines()
                        .filter(|line| !line.starts_with("//"))
                        .join("\n"),
                    Some(node) => bail!("Expected a code chunk, got {}", node.to_string()),
                    None => bail!("Expect a code chunk, got `None`"),
                }
            };

            let expected = parts.next().unwrap_or_default().trim();
            if !update {
                assert_eq!(
                    actual, expected,
                    "\n\nFile: {}\nDocsQL: {}",
                    filename, docsql
                );

                if env::var_os("NO_HTTP").is_none()
                    && env::var_os("CI").is_none() // Don't run HTTP requests on CI yet
                    && actual.starts_with("GET ")
                {
                    let url = actual.strip_prefix("GET ").unwrap_or(&actual);
                    if filename.ends_with(".openalex") {
                        validate_openalex_url(url).await?;
                    } else if filename.ends_with(".github") {
                        validate_github_url(url).await?;
                    }
                }
            } else {
                *test = format!("{docsql}\n---\n{actual}\n");
            }
        }

        if update {
            let mut contents = tests.join("\n\n").trim_end().to_string();
            contents.push('\n');
            write(path, contents)?;
        }
    }

    Ok(())
}

/// Validate an OpenAlex URL by making an HTTP GET request
async fn validate_openalex_url(url: &str) -> Result<()> {
    let client = Client::new();

    // Add a user agent to be respectful to the OpenAlex API
    let response = client
        .get(url)
        .header(USER_AGENT, STENCILA_USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        bail!("OpenAlex API request `{url}` failed with status {status}:\n\n{body}");
    }

    Ok(())
}

/// Validate a GitHub URL by making an HTTP GET request
async fn validate_github_url(url: &str) -> Result<()> {
    let client = Client::new();

    // Build request with appropriate headers for GitHub API
    let mut request = client.get(url).header(USER_AGENT, STENCILA_USER_AGENT);

    // Add GitHub token if available for authentication
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        bail!("GitHub API request `{url}` failed with status {status}:\n\n{body}");
    }

    Ok(())
}
