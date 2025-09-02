use std::{
    env,
    fs::{read_to_string, write},
    path::PathBuf,
};

use eyre::{Result, bail};
use glob::glob;
use itertools::Itertools;
use pretty_assertions::assert_eq;
use stencila_kernel_docsql::DocsQLKernelInstance;
use stencila_kernel_jinja::stencila_kernel::{
    KernelInstance,
    stencila_schema::{CodeChunk, Node, Null},
};
use tokio::{
    self,
    sync::{mpsc, watch},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn golden() -> Result<()> {
    let (.., receiver) = watch::channel(Node::Null(Null));
    let (sender, ..) = mpsc::channel(1);
    let mut kernel = DocsQLKernelInstance::new(None, Some((receiver, sender)))?;
    kernel.start_here().await?;

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
            .chain(glob(&(pattern.clone() + "/*.github"))?.flatten())
            .chain(glob(&(pattern + "/*.zenodo"))?.flatten())
            .collect()
    };

    let update = env::var("UPDATE_GOLDEN").is_ok();
    let github_auth = stencila_secrets::env_or_get("GITHUB_TOKEN").ok();

    for path in test_paths {
        let contents = read_to_string(&path)?;

        let Some(filename) = path.file_name() else {
            continue;
        };
        let filename = filename.to_string_lossy();

        let mut tests = contents.split("\n\n").map(String::from).collect_vec();
        for (test_index, test) in tests.iter_mut().enumerate() {
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

                // For API queries, also execute the query and save results to JSON when explicitly requested
                if env::var("EXECUTE_QUERIES").is_ok()
                    && actual.starts_with("GET ")
                    && (filename.ends_with(".openalex")
                        || filename.ends_with(".zenodo")
                        || (filename.ends_with(".github")
                            && (!actual.starts_with("GET https://api.github.com/search/code")
                                || github_auth.is_some())))
                {
                    let (node_outputs, node_messages) = kernel.execute(docsql).await?;

                    if node_messages.is_empty() && !node_outputs.is_empty() {
                        // Create JSON filename from test file name, query type, and index
                        let (base_name, query_type) =
                            if let Some(base) = filename.strip_suffix(".openalex") {
                                (base, "openalex")
                            } else if let Some(base) = filename.strip_suffix(".github") {
                                (base, "github")
                            } else if let Some(base) = filename.strip_suffix(".zenodo") {
                                (base, "zenodo")
                            } else {
                                (filename.as_ref(), "unknown")
                            };

                        let json_filename =
                            format!("{}-{}-{}.json", base_name, query_type, test_index + 1);
                        let json_path = path.with_file_name(json_filename);

                        // Convert nodes to JSON and save
                        let json_content = serde_json::to_string_pretty(&node_outputs)?;
                        write(&json_path, json_content)?;
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
