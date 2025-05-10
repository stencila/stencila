use std::{
    env,
    fs::{read_to_string, write},
    path::PathBuf,
};

use common_dev::pretty_assertions::assert_eq;
use kernel_docsql::DocsQLKernelInstance;
use kernel_jinja::kernel::{
    common::{
        eyre::{bail, Result},
        glob::glob,
        itertools::Itertools,
        tokio,
    },
    schema::{CodeChunk, Node},
    KernelInstance,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn golden() -> Result<()> {
    let update = env::var("UPDATE_GOLDEN").is_ok();

    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/*.cypher";

    let mut kernel = DocsQLKernelInstance::new(None, None)?;

    for path in glob(&pattern)?.flatten() {
        let contents = read_to_string(&path)?;

        let mut tests = contents.split("\n\n").map(String::from).collect_vec();
        for test in tests.iter_mut() {
            let mut parts = test.split("---");

            let docsql = parts.next().unwrap_or_default().trim();
            if docsql.is_empty() {
                continue;
            }

            let (outputs, messages) = kernel.execute(&format!("{docsql}.explain()")).await?;
            assert_eq!(messages, []);

            let actual = match outputs.first() {
                Some(Node::CodeChunk(CodeChunk { code, .. })) => code.lines().skip(1).join("\n"),
                Some(node) => bail!("Expected a code chunk, got {}", node.to_string()),
                None => bail!("Expect a code chunk, got `None`"),
            };

            let expected = parts.next().unwrap_or_default().trim();
            if !update {
                assert_eq!(
                    actual,
                    expected,
                    "\n\nFile: {}\nDocsQL: {}",
                    path.file_name().unwrap().to_string_lossy(),
                    docsql
                )
            } else {
                *test = format!("{docsql}\n---\n{actual}\n");
            }
        }

        if update {
            let contents = tests.join("\n\n");
            write(path, contents)?;
        }
    }

    Ok(())
}
