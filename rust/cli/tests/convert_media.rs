//! Exercise the media contract through the installed command's argument path.
use eyre::{OptionExt, Result};
use serde_json::Value;
use std::{path::Path, process::Command};

fn convert(dir: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_stencila"))
        .arg("convert")
        .args(args)
        .current_dir(dir)
        .env("STENCILA_UPGRADE_INTERVAL", "never")
        .env("XDG_CACHE_HOME", dir.join(".cache"))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn values<'a>(value: &'a Value, key: &str) -> Vec<&'a str> {
    match value {
        Value::Object(object) => object
            .iter()
            .flat_map(|(name, value)| {
                if name == key {
                    value.as_str().into_iter().collect()
                } else {
                    values(value, key)
                }
            })
            .collect(),
        Value::Array(array) => array.iter().flat_map(|value| values(value, key)).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn jats_conversion_rebases_each_output_and_copies_originals() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../codec-jats/tests/media/PMC5754353.1");
    let source = dir.path().join("source");
    std::fs::create_dir(&source)?;
    for entry in std::fs::read_dir(fixtures)? {
        let entry = entry?;
        std::fs::copy(entry.path(), source.join(entry.file_name()))?;
    }
    convert(
        dir.path(),
        &[
            "source/PMC5754353.1.xml",
            "one/out.json",
            "two/deeper/out.json",
        ],
    )?;
    for out in ["one/out.json", "two/deeper/out.json"] {
        let path = dir.path().join(out);
        let value: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        let references = values(&value, "contentUrl");
        assert_eq!(references.len(), 2);
        for reference in references {
            let found = path.parent().ok_or_eyre("parent")?.join(reference);
            assert!(found.is_file(), "{reference}");
            assert_eq!(
                std::fs::read(&found)?,
                std::fs::read(source.join(found.file_name().ok_or_eyre("filename")?))?
            );
        }
        for target in values(&value, "target")
            .into_iter()
            .filter(|target| target.ends_with(".pdf"))
        {
            assert!(path.parent().ok_or_eyre("parent")?.join(target).is_file());
        }
    }
    convert(
        dir.path(),
        &[
            "source/PMC5754353.1.xml",
            "copied/out.json",
            "--extract-media",
            "originals",
        ],
    )?;
    let value: Value = serde_json::from_slice(&std::fs::read(dir.path().join("copied/out.json"))?)?;
    for reference in values(&value, "contentUrl") {
        assert!(reference.starts_with("../originals/"));
        let copied = std::fs::read(dir.path().join("copied").join(reference))?;
        assert!(
            std::fs::read_dir(&source)?
                .filter_map(Result::ok)
                .any(|entry| std::fs::read(entry.path()).is_ok_and(|bytes| bytes == copied))
        );
    }
    Ok(())
}

#[test]
fn nih_missing_media_are_machine_readable_losses() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join("article.xml"),
        include_bytes!("../../codec-jats/tests/media/nih-manuscript/article.xml"),
    )?;
    convert(
        dir.path(),
        &["article.xml", "out.json", "--input-losses", "losses.json"],
    )?;
    let losses: Value = serde_json::from_slice(&std::fs::read(dir.path().join("losses.json"))?)?;
    assert_eq!(
        losses
            .as_object()
            .ok_or_eyre("expected losses object")?
            .len(),
        3
    );
    assert_eq!(
        losses["//article[1]/floats-group[1]/fig[1]/graphic[1]/@xlink:href (unresolved: nihms-1836442-f0001.jpg)"],
        1
    );
    Ok(())
}

#[test]
fn meca_cli_defaults_to_original_files_and_honors_embedding() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join("article.meca"),
        include_bytes!("../../codec-meca/tests/fixtures/originals.meca"),
    )?;
    convert(dir.path(), &["article.meca", "output/out.json"])?;
    let path = dir.path().join("output/out.json");
    let value: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    for reference in values(&value, "contentUrl") {
        let found = path.parent().ok_or_eyre("parent")?.join(reference);
        assert_eq!(
            std::fs::read(found)?,
            include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif")
        );
    }
    convert(
        dir.path(),
        &[
            "article.meca",
            "embedded.json",
            "--embed-media",
            "--embed-supplements",
        ],
    )?;
    let value: Value = serde_json::from_slice(&std::fs::read(dir.path().join("embedded.json"))?)?;
    assert!(
        values(&value, "contentUrl")
            .iter()
            .any(|reference| reference.starts_with("data:image/png;base64,"))
    );
    assert!(
        !values(&value, "type")
            .into_iter()
            .all(|typ| typ != "Datatable")
    );
    Ok(())
}

#[test]
fn meca_to_markdown_links_resolve_beside_and_away_from_input() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join("article.meca"),
        include_bytes!("../../codec-meca/tests/fixtures/originals.meca"),
    )?;
    for out in ["article.md", "output/article.md"] {
        convert(dir.path(), &["article.meca", out])?;
        let path = dir.path().join(out);
        let markdown = std::fs::read_to_string(&path)?;
        // Image links only: an embedded supplement's target still names the
        // file in the archive (a known, separate limitation)
        let links = markdown
            .split("![")
            .skip(1)
            .filter_map(|rest| rest.split_once("]("))
            .filter_map(|(_, rest)| rest.split(')').next())
            .collect::<Vec<_>>();
        assert!(!links.is_empty(), "no links in {out}: {markdown}");
        for link in links {
            assert!(
                path.parent().ok_or_eyre("parent")?.join(link).is_file(),
                "{out}: unresolved {link}"
            );
        }
    }
    Ok(())
}
