use std::{env, path::PathBuf};

use glob::glob;
use pretty_assertions::assert_eq;
use stencila_svg_components::{compile, diagnostics::MessageLevel};

/// Test compilation of all fixture SVG files.
///
/// For each `tests/fixtures/**/*.svg` file, compiles it and compares the
/// output against sibling files:
///
/// - `{name}.compiled.svg` — the compiled SVG (or "PASS_THROUGH" if no
///   `s:` elements were found)
/// - `{name}.messages` — diagnostic messages (only if non-empty)
///
/// If the expected files don't exist yet, they are created. Set
/// `UPDATE_SNAPSHOTS=true` to overwrite existing files:
///
/// ```sh
/// UPDATE_SNAPSHOTS=true cargo test -p stencila-svg-components
/// ```
#[test]
#[allow(clippy::print_stderr)]
fn examples() {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/**/*.svg")
        .to_string_lossy()
        .to_string();

    let update = env::var("UPDATE_SNAPSHOTS").unwrap_or_default() == "true";

    let fixtures: Vec<PathBuf> = glob(&pattern)
        .expect("valid glob pattern")
        .flatten()
        // Skip .compiled.svg files
        .filter(|p| {
            !p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(".compiled.")
        })
        .collect();

    assert!(!fixtures.is_empty(), "no fixture files found");

    let mut failures = Vec::new();

    for fixture_path in &fixtures {
        let name = fixture_path
            .file_stem()
            .expect("fixture has file stem")
            .to_string_lossy()
            .to_string();
        let dir = fixture_path.parent().expect("fixture has parent dir");

        let source = std::fs::read_to_string(fixture_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", fixture_path.display()));
        let source = source.trim();

        let result = compile(source);

        // --- Compiled SVG ---

        let compiled_file = dir.join(format!("{name}.compiled.svg"));
        let actual_compiled = match &result.compiled {
            Some(svg) => svg.clone(),
            None => "PASS_THROUGH".to_string(),
        };

        // Validate compiled SVG is well-formed XML (catches duplicate attributes, etc.)
        if let Some(svg) = &result.compiled
            && let Err(err) = roxmltree::Document::parse(svg)
        {
            failures.push(format!("{}: invalid XML: {err}", fixture_path.display()));
            eprintln!("--- INVALID XML: {} ---\n{err}", fixture_path.display());
        }

        if compiled_file.exists() {
            let expected =
                std::fs::read_to_string(&compiled_file).expect("failed to read compiled file");
            let expected = expected.trim_end().to_string();

            if actual_compiled != expected {
                if update {
                    std::fs::write(&compiled_file, &actual_compiled)
                        .expect("failed to write compiled file");
                } else {
                    failures.push(format!("{}: compiled SVG differs", fixture_path.display()));
                    // Use pretty_assertions for the diff output
                    eprintln!("--- FAIL: {} ---", fixture_path.display());
                    // This will panic with a nice diff but we catch it below
                    let _ = std::panic::catch_unwind(|| {
                        assert_eq!(actual_compiled, expected, "{}", fixture_path.display());
                    });
                }
            }
        } else {
            std::fs::write(&compiled_file, &actual_compiled)
                .expect("failed to write compiled file");
        }

        // --- Messages ---

        let messages_file = dir.join(format!("{name}.messages"));
        let actual_messages: String = result
            .messages
            .iter()
            .map(|m| {
                let level = match m.level {
                    MessageLevel::Error => "error",
                    MessageLevel::Warning => "warning",
                };
                format!("{level}: {}", m.message)
            })
            .collect::<Vec<_>>()
            .join("\n");

        if actual_messages.is_empty() {
            // Remove stale messages file if present
            if messages_file.exists() {
                if update {
                    std::fs::remove_file(&messages_file).expect("failed to remove messages file");
                } else {
                    failures.push(format!(
                        "{}: messages file exists but no messages produced",
                        fixture_path.display()
                    ));
                }
            }
        } else if messages_file.exists() {
            let expected =
                std::fs::read_to_string(&messages_file).expect("failed to read messages file");
            let expected = expected.trim_end().to_string();

            if actual_messages != expected {
                if update {
                    std::fs::write(&messages_file, &actual_messages)
                        .expect("failed to write messages file");
                } else {
                    failures.push(format!("{}: messages differ", fixture_path.display()));
                    let _ = std::panic::catch_unwind(|| {
                        assert_eq!(
                            actual_messages,
                            expected,
                            "{} messages",
                            fixture_path.display()
                        );
                    });
                }
            }
        } else {
            std::fs::write(&messages_file, &actual_messages)
                .expect("failed to write messages file");
        }
    }

    assert!(
        failures.is_empty(),
        "fixture failures:\n  {}",
        failures.join("\n  ")
    );
}
