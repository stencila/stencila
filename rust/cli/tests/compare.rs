//! Process-level tests for the `stencila compare` command
//!
//! These run the built binary, rather than calling into the library, because the
//! contract being tested is the process exit status: 0 for equal, 1 for different,
//! and 2 for any failure.

use std::{
    fs::{read_to_string, write},
    path::Path,
    process::{Command, Output},
};

use eyre::{Result, bail};
use tempfile::TempDir;

/// Run `stencila compare` with the given arguments
fn compare<I, S>(dir: &Path, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_stencila"))
        .arg("compare")
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        // Avoid an upgrade check writing to stderr during tests
        .env("STENCILA_UPGRADE_INTERVAL", "never")
        .output()
        .expect("Unable to run the `stencila` binary")
}

/// The exit code of a process, requiring that it exited rather than being signalled
fn code(output: &Output) -> Result<i32> {
    match output.status.code() {
        Some(code) => Ok(code),
        None => bail!("The process did not exit normally"),
    }
}

/// A workspace containing two documents that differ
fn workspace() -> Result<TempDir> {
    let dir = tempfile::tempdir()?;
    write(
        dir.path().join("before.smd"),
        "# Methods\n\nOne two three.\n",
    )?;
    write(dir.path().join("after.smd"), "# Method\n\nOne two three.\n")?;
    write(dir.path().join("copy.smd"), "# Methods\n\nOne two three.\n")?;
    Ok(dir)
}

#[test]
fn equal_documents_exit_zero() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "copy.smd"]);
    assert_eq!(code(&output)?, 0);

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with("equal\n"), "{stdout}");

    Ok(())
}

#[test]
fn a_document_equals_itself() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "before.smd"]);
    assert_eq!(code(&output)?, 0);

    Ok(())
}

#[test]
fn different_documents_exit_one() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "after.smd"]);
    assert_eq!(code(&output)?, 1);

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with("different\n"), "{stdout}");
    assert!(stdout.contains("~ value"), "{stdout}");

    Ok(())
}

#[test]
fn different_documents_still_write_their_output() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "after.smd", "comparison.json"]);
    assert_eq!(code(&output)?, 1);
    assert!(output.stdout.is_empty());

    // The artifact is created, and is the comparison itself
    let json = read_to_string(dir.path().join("comparison.json"))?;
    assert!(json.ends_with("\n"));
    let value: serde_json::Value = serde_json::from_str(&json)?;
    let keys: Vec<&str> = value
        .as_object()
        .expect("Expected an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["formatVersion", "alignment", "differences"]);

    Ok(())
}

#[test]
fn machine_output_has_no_styling_or_prose() -> Result<()> {
    let dir = workspace()?;

    for (format, first) in [("json", '{'), ("yaml", 'f')] {
        let output = compare(dir.path(), ["before.smd", "after.smd", "--to", format]);
        assert_eq!(code(&output)?, 1);

        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(stdout.chars().next(), Some(first), "{stdout}");
        assert!(stdout.ends_with('\n'), "{stdout}");
        assert!(!stdout.ends_with("\n\n"), "{stdout}");
        assert!(!stdout.contains('\u{1b}'), "{stdout}");
    }

    Ok(())
}

#[test]
fn yaml_is_inferred_from_the_output_extension() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "after.smd", "comparison.yaml"]);
    assert_eq!(code(&output)?, 1);

    let yaml = read_to_string(dir.path().join("comparison.yaml"))?;
    assert!(yaml.starts_with("formatVersion:"), "{yaml}");

    Ok(())
}

#[test]
fn html_is_inferred_from_the_output_extension() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "after.smd", "comparison.html"]);
    assert_eq!(code(&output)?, 1);
    // Writing the page is not viewing it, so nothing is announced or printed
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.is_empty());

    let html = read_to_string(dir.path().join("comparison.html"))?;
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(html.contains("<td><span class=\"subject\">"), "{html}");

    Ok(())
}

/// The `BROWSER` environment variable is only honored by `webbrowser` on Linux, so
/// elsewhere this test would open a real browser window
#[cfg(target_os = "linux")]
#[test]
fn the_view_option_writes_a_page_and_reports_where() -> Result<()> {
    use std::process::Command;

    let dir = workspace()?;

    let output = Command::new(env!("CARGO_BIN_EXE_stencila"))
        .args(["compare", "before.smd", "after.smd", "--view"])
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .env("STENCILA_UPGRADE_INTERVAL", "never")
        // A browser that does nothing, successfully
        .env("BROWSER", "true")
        .output()?;

    // Documents that differ still differ when viewed
    assert_eq!(code(&output)?, 1);
    // The report went to the page, not to stdout
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr)?;
    let url = stderr
        .split_whitespace()
        .find(|word| word.starts_with("file://"))
        .unwrap_or_else(|| panic!("Expected the view URL in: {stderr}"));

    let path = Path::new(url.trim_start_matches("file://"));
    let html = read_to_string(path)?;
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    // Not inside the workspace, and not left for the caller to find by guessing
    assert!(!path.starts_with(dir.path()));

    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn viewing_a_machine_format_is_an_error() -> Result<()> {
    let dir = workspace()?;

    let output = compare(
        dir.path(),
        ["before.smd", "after.smd", "--view", "--to", "json"],
    );
    assert_eq!(code(&output)?, 2);
    assert!(output.stdout.is_empty());

    let output = compare(
        dir.path(),
        ["before.smd", "after.smd", "comparison.yaml", "--view"],
    );
    assert_eq!(code(&output)?, 2);
    assert!(!dir.path().join("comparison.yaml").exists());

    Ok(())
}

#[test]
fn cross_format_comparison_is_supported() -> Result<()> {
    let dir = workspace()?;
    write(
        dir.path().join("before.json"),
        serde_json::to_string(&serde_json::json!({
            "type": "Article",
            "content": [{
                "type": "Heading",
                "level": 1,
                "content": [{"type": "Text", "value": "Methods"}]
            }, {
                "type": "Paragraph",
                "content": [{"type": "Text", "value": "One two three."}]
            }]
        }))?,
    )?;

    let output = compare(dir.path(), ["before.smd", "before.json"]);
    assert_eq!(
        code(&output)?,
        0,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[test]
fn the_summary_option_stops_after_the_counts() -> Result<()> {
    let dir = workspace()?;

    let output = compare(dir.path(), ["before.smd", "after.smd", "--summary"]);
    assert_eq!(code(&output)?, 1);

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("differences: 1"), "{stdout}");
    assert!(!stdout.contains("~ value"), "{stdout}");

    Ok(())
}

#[test]
fn usage_errors_exit_two() -> Result<()> {
    let dir = workspace()?;

    // A missing input
    let output = compare(dir.path(), ["before.smd", "missing.smd"]);
    assert_eq!(code(&output)?, 2);

    // A summary of a machine format
    let output = compare(
        dir.path(),
        ["before.smd", "after.smd", "--summary", "--to", "json"],
    );
    assert_eq!(code(&output)?, 2);

    // An unknown output extension
    let output = compare(dir.path(), ["before.smd", "after.smd", "comparison.toml"]);
    assert_eq!(code(&output)?, 2);
    assert!(!dir.path().join("comparison.toml").exists());

    // An output that is one of the inputs
    let output = compare(dir.path(), ["before.smd", "after.smd", "before.smd"]);
    assert_eq!(code(&output)?, 2);
    assert_eq!(
        read_to_string(dir.path().join("before.smd"))?,
        "# Methods\n\nOne two three.\n"
    );

    // An exhausted alignment budget
    let output = compare(
        dir.path(),
        ["before.smd", "after.smd", "--alignment-cell-budget", "0"],
    );
    assert_eq!(code(&output)?, 2);

    Ok(())
}

#[test]
fn input_losses_are_labelled_by_side() -> Result<()> {
    let dir = workspace()?;
    // An element that the JATS codec does not understand is a decoding loss
    write(
        dir.path().join("lossy.jats"),
        "<article><body><sec><title>Methods</title><p>One two three.</p>\
         <unknown-thing>x</unknown-thing></sec></body></article>",
    )?;

    // Warning continues, and says which side the losses came from
    let output = compare(
        dir.path(),
        ["before.smd", "lossy.jats", "--input-losses", "warn"],
    );
    assert_eq!(code(&output)?, 1);
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("right document"), "{stderr}");
    assert!(stderr.contains("unknown-thing"), "{stderr}");

    // The same losses on the left are labelled as such
    let output = compare(
        dir.path(),
        ["lossy.jats", "before.smd", "--input-losses", "warn"],
    );
    assert_eq!(code(&output)?, 1);
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("left document"), "{stderr}");

    // Ignoring is silent
    let output = compare(
        dir.path(),
        ["before.smd", "lossy.jats", "--input-losses", "ignore"],
    );
    assert_eq!(code(&output)?, 1);
    let stderr = String::from_utf8(output.stderr)?;
    assert!(!stderr.contains("unknown-thing"), "{stderr}");

    // Aborting produces no comparison
    let output = compare(
        dir.path(),
        ["before.smd", "lossy.jats", "--input-losses", "abort"],
    );
    assert_eq!(code(&output)?, 2);
    assert!(output.stdout.is_empty());

    Ok(())
}
