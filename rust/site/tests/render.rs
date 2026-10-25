//! Full rendering uses a process-global configuration singleton initialized
//! from the working directory. Keep these checks in a single integration test
//! so route discovery uses the temporary site instead of the repository's site.

use std::{env, fs, path::Path, process::Command};

use eyre::Result;
use tempfile::TempDir;

use stencila_codec::DecodeOptions;
use stencila_format::Format;
use stencila_site::render;

#[tokio::test]
async fn specimen_rendering_in_isolated_workspace() -> Result<()> {
    let source_dir = TempDir::new()?;
    let output_dir = TempDir::new()?;
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .arg(source_dir.path())
        .status()?;
    assert!(status.success(), "failed to initialize fixture repository");
    fs::write(
        source_dir.path().join("stencila.toml"),
        r#"[site]
root = "."

[site.specimen.layout]
preset = "docs"
left-sidebar = false
"#,
    )?;
    fs::write(
        source_dir.path().join("index.md"),
        "# Hello\n\nMinimal page.\n",
    )?;

    let previous_dir = env::current_dir()?;
    env::set_current_dir(source_dir.path())?;
    let result = check_specimen_rendering(source_dir.path(), output_dir.path()).await;
    env::set_current_dir(previous_dir)?;
    result
}

async fn check_specimen_rendering(source_dir: &Path, output_dir: &Path) -> Result<()> {
    let source_files = [source_dir.join("index.md")];
    let specimen_path = output_dir.join("_specimen/index.html");

    // Check both a full render and an incremental render after removing the
    // specimen. It has no source file, so it must be regenerated in both cases.
    for source_filter in [None, Some(source_files.as_slice())] {
        if specimen_path.exists() {
            fs::remove_file(&specimen_path)?;
        }

        let result = render(
            source_dir,
            output_dir,
            "https://localhost",
            None,
            None,
            None,
            source_filter,
            None,
            |doc_path, _arguments| async move {
                let content = tokio::fs::read_to_string(&doc_path).await?;
                stencila_codecs::from_str(
                    &content,
                    Some(DecodeOptions {
                        format: Some(Format::Markdown),
                        ..Default::default()
                    }),
                )
                .await
            },
        )
        .await?;

        assert!(result.documents_failed.is_empty());
        assert_eq!(
            result.documents_ok,
            vec![(source_files[0].clone(), "/".to_string())],
            "render should discover only the fixture document"
        );

        let html = fs::read_to_string(&specimen_path)?;
        for heading in ["typography", "code", "tables"] {
            assert!(
                html.contains(&format!("id={heading}")),
                "specimen should contain the {heading} heading"
            );
        }
        assert!(html.contains("<stencila-layout"));
        assert!(
            !html.contains("<stencila-left-sidebar"),
            "specimen should use the fixture's layout override"
        );
    }

    Ok(())
}
