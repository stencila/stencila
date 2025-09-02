use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, eyre};
use tempfile::TempDir;

use stencila_tools::cli::Cli;
use stencila_tools::{ToolType, detect_managers};

/// Helper function to get the path to example workspaces
fn example_workspace_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("should exist")
        .parent()
        .expect("should exist")
        .join("examples")
        .join("workspaces")
        .join(name)
}

/// Helper function to test language dependency detection
fn has_language_dependencies_helper(path: &std::path::Path) -> bool {
    let python_files = [path.join("pyproject.toml"), path.join("requirements.txt")];

    let r_files = [path.join("renv.lock"), path.join("DESCRIPTION")];

    python_files.iter().any(|p| p.exists()) || r_files.iter().any(|p| p.exists())
}

// Pro-tip! To get logs for some of these tests use:
//
// ```sh
// RUST_LOG=trace cargo test -p tools -- --nocapture
// ```

/// Test install command with dry-run on all example workspaces
///
/// This test simply verifies that the install command does not crash when run
/// with --dry-run (no tool specified) on various example workspace configurations.
#[test_log::test(tokio::test)]
async fn test_install_dry_run_all_workspaces() -> Result<()> {
    let workspace_names = [
        "no-dependencies",
        "pyproject-toml",
        "requirements-txt",
        "devbox-python-r",
        "mise-toml",
        "r-renv",
        "r-description",
    ];

    for workspace_name in workspace_names {
        let workspace_path = example_workspace_path(workspace_name);
        assert!(
            workspace_path.exists(),
            "Workspace {workspace_name} should exist"
        );

        let cli = Cli::try_parse_from([
            "stencila",
            "install",
            "--dry-run",
            "-C",
            workspace_path.to_str().expect("to be ok"),
        ])?;

        // Should not crash
        cli.run().await.map_err(|e| {
            // Provide context about which workspace failed
            eyre!("Install failed for workspace {}: {}", workspace_name, e)
        })?;
    }

    Ok(())
}

/// Test install command with skip flags
#[test_log::test(tokio::test)]
async fn test_install_with_skip_flags() -> Result<()> {
    let workspace_path = example_workspace_path("pyproject-toml");

    // Test skip-python flag
    let cli = Cli::try_parse_from([
        "stencila",
        "install",
        "--dry-run",
        "--skip-python",
        "-C",
        workspace_path.to_str().expect("to be ok"),
    ])?;
    cli.run().await?;

    // Test skip-r flag
    let cli = Cli::try_parse_from([
        "stencila",
        "install",
        "--dry-run",
        "--skip-r",
        "-C",
        workspace_path.to_str().expect("to be ok"),
    ])?;
    cli.run().await?;

    // Test skip-env flag
    let cli = Cli::try_parse_from([
        "stencila",
        "install",
        "--dry-run",
        "--skip-env",
        "-C",
        workspace_path.to_str().expect("to be ok"),
    ])?;
    cli.run().await?;

    Ok(())
}

/// Test install command with force flag
#[test_log::test(tokio::test)]
async fn test_install_with_force_flag() -> Result<()> {
    let workspace_path = example_workspace_path("no-dependencies");

    let cli = Cli::try_parse_from([
        "stencila",
        "install",
        "--dry-run",
        "--force",
        "-C",
        workspace_path.to_str().expect("to be ok"),
    ])?;

    cli.run().await?;
    Ok(())
}

/// Test install command with non-existent directory
#[test_log::test(tokio::test)]
async fn test_install_non_existent_directory() -> Result<()> {
    let cli = Cli::try_parse_from(["stencila", "install", "-C", "/path/that/does/not/exist"])?;

    let result = cli.run().await;
    assert!(
        result.is_err(),
        "Install should fail for non-existent directory"
    );

    Ok(())
}

/// Test that mise.toml detection works correctly
#[test]
fn test_detect_mise_manager() {
    let workspace_path = example_workspace_path("mise-toml");
    let managers = detect_managers(&workspace_path, &[ToolType::Environments]);

    assert_eq!(
        managers.len(),
        1,
        "Should detect exactly one environment manager"
    );
    let (manager, config_path) = &managers[0];
    assert_eq!(manager.name(), "mise", "Should detect mise as the manager");
    assert!(
        config_path.ends_with("mise.toml"),
        "Config path should be mise.toml"
    );
}

/// Test that devbox.json detection works correctly
#[test]
fn test_detect_devbox_manager() -> Result<()> {
    let workspace_path = example_workspace_path("devbox-python-r");
    let managers = detect_managers(&workspace_path, &[ToolType::Environments]);

    // Should detect at least devbox, might also detect mise from parent directory
    assert!(
        !managers.is_empty(),
        "Should detect at least one environment manager"
    );
    let devbox_manager = managers.iter().find(|(m, _)| m.name() == "devbox");
    assert!(
        devbox_manager.is_some(),
        "Should detect devbox as a manager"
    );

    let (_, config_path) = devbox_manager.expect("to be ok");
    assert!(
        config_path.ends_with("devbox.json"),
        "Config path should be devbox.json"
    );

    Ok(())
}

/// Test that environment manager detection from parent directories works correctly
#[test]
fn test_detect_managers_from_parent() {
    let workspace_path = example_workspace_path("no-dependencies");
    let managers = detect_managers(&workspace_path, &[ToolType::Environments]);

    // The no-dependencies workspace is inside the project, so it should detect mise from parent
    assert!(
        !managers.is_empty(),
        "Should detect environment managers from parent directories"
    );
    let mise_manager = managers.iter().find(|(m, _)| m.name() == "mise");
    assert!(
        mise_manager.is_some(),
        "Should detect mise from parent directory"
    );
}

/// Test language dependency detection for Python projects
#[test]
fn test_language_dependency_detection_python() {
    // Test pyproject.toml detection
    let workspace_path = example_workspace_path("pyproject-toml");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "Should detect pyproject.toml as language dependency"
    );

    // Test requirements.txt detection
    let workspace_path = example_workspace_path("requirements-txt");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "Should detect requirements.txt as language dependency"
    );
}

/// Test language dependency detection for R projects
#[test]
fn test_language_dependency_detection_r() {
    // Test renv.lock detection
    let workspace_path = example_workspace_path("r-renv");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "Should detect renv.lock as language dependency"
    );

    // Test DESCRIPTION detection
    let workspace_path = example_workspace_path("r-description");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "Should detect DESCRIPTION as language dependency"
    );
}

/// Test that no language dependencies are detected in empty workspace
#[test]
fn test_no_language_dependencies() {
    let workspace_path = example_workspace_path("no-dependencies");
    assert!(
        !has_language_dependencies_helper(&workspace_path),
        "Should detect no language dependencies in empty workspace"
    );
}

/// Test install behavior in workspace with multiple config types
#[test]
fn test_mixed_workspace_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create multiple config files
    std::fs::write(
        temp_path.join("pyproject.toml"),
        "[project]\nname = \"test\"",
    )?;
    std::fs::write(temp_path.join("mise.toml"), "[tools]\npython = \"3.11\"")?;

    // Test environment manager detection
    let env_managers = detect_managers(temp_path, &[ToolType::Environments]);
    assert_eq!(
        env_managers.len(),
        1,
        "Should detect mise environment manager"
    );
    assert_eq!(env_managers[0].0.name(), "mise");

    // Test language dependency detection
    assert!(
        has_language_dependencies_helper(temp_path),
        "Should detect Python dependencies"
    );

    Ok(())
}

/// Test language dependency detection for determining setup behavior
#[test]
fn test_language_dependency_detection_logic() {
    // Empty workspace has no language dependencies
    let workspace_path = example_workspace_path("no-dependencies");
    assert!(
        !has_language_dependencies_helper(&workspace_path),
        "Empty workspace should have no language dependencies"
    );

    // Workspace with Python deps has language dependencies
    let workspace_path = example_workspace_path("pyproject-toml");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "Python workspace should have language dependencies"
    );

    // Workspace with R deps has language dependencies
    let workspace_path = example_workspace_path("r-renv");
    assert!(
        has_language_dependencies_helper(&workspace_path),
        "R workspace should have language dependencies"
    );
}

/// Test installing multiple tools at once
#[test_log::test(tokio::test)]
async fn test_install_multiple_tools() -> Result<()> {
    // Test installing multiple tools with dry-run
    let cli = Cli::try_parse_from([
        "stencila",
        "install",
        "mise",
        "uv",
        "ruff",
        "--dry-run",
        "--force",
    ])?;

    // Should not crash when installing multiple tools
    let result = cli.run().await;

    // The install might fail if tools don't support dry-run mode, but it shouldn't crash
    // We just check that the command parsing and basic execution works
    match result {
        Ok(_) => {
            // Success - tools support dry-run or were already installed
        }
        Err(e) => {
            // Failure is expected for tools that don't support dry-run
            // but we still want to ensure the command structure works
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("No tool with name"),
                "Should recognize all tool names: {error_msg}"
            );
        }
    }

    Ok(())
}

/// Test comprehensive setup plan generation
#[test]
fn test_comprehensive_setup_plan() {
    // Test mise workspace: should detect mise, no language deps
    let workspace_path = example_workspace_path("mise-toml");
    let env_managers = detect_managers(&workspace_path, &[ToolType::Environments]);
    let has_lang_deps = has_language_dependencies_helper(&workspace_path);

    assert_eq!(env_managers.len(), 1, "Should detect mise");
    assert_eq!(env_managers[0].0.name(), "mise");
    assert!(!has_lang_deps, "Should have no language deps");

    // Test pyproject.toml workspace: should detect mise from parent AND process pyproject.toml
    let workspace_path = example_workspace_path("pyproject-toml");
    let env_managers = detect_managers(&workspace_path, &[ToolType::Environments]);
    let has_lang_deps = has_language_dependencies_helper(&workspace_path);

    // Note: this detects mise from parent directory (the project root has mise.toml)
    assert!(
        !env_managers.is_empty(),
        "Should detect environment manager from parent"
    );
    assert!(has_lang_deps, "Should detect pyproject.toml");

    // Test R workspace: should detect R deps
    let workspace_path = example_workspace_path("r-renv");
    let has_lang_deps = has_language_dependencies_helper(&workspace_path);

    assert!(has_lang_deps, "Should detect renv.lock");
}

/// Test that detection works correctly for different file types
#[test]
fn test_file_type_detection_specificity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Test each Python file type individually
    std::fs::write(
        temp_path.join("pyproject.toml"),
        "[project]\nname = \"test\"",
    )?;
    assert!(
        has_language_dependencies_helper(temp_path),
        "Should detect pyproject.toml"
    );
    std::fs::remove_file(temp_path.join("pyproject.toml"))?;

    std::fs::write(temp_path.join("requirements.txt"), "requests")?;
    assert!(
        has_language_dependencies_helper(temp_path),
        "Should detect requirements.txt"
    );
    std::fs::remove_file(temp_path.join("requirements.txt"))?;

    // Test each R file type individually
    std::fs::write(temp_path.join("renv.lock"), "{}")?;
    assert!(
        has_language_dependencies_helper(temp_path),
        "Should detect renv.lock"
    );
    std::fs::remove_file(temp_path.join("renv.lock"))?;

    std::fs::write(temp_path.join("DESCRIPTION"), "Package: test")?;
    assert!(
        has_language_dependencies_helper(temp_path),
        "Should detect DESCRIPTION"
    );
    std::fs::remove_file(temp_path.join("DESCRIPTION"))?;

    // Verify empty directory has no deps
    assert!(
        !has_language_dependencies_helper(temp_path),
        "Empty directory should have no deps"
    );

    Ok(())
}

/// Test environment manager detection with multiple configs
#[test]
fn test_multiple_environment_managers() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create configs for multiple environment managers
    std::fs::write(temp_path.join("mise.toml"), "[tools]\npython = \"3.11\"")?;
    std::fs::write(
        temp_path.join("devbox.json"),
        r#"{"packages": ["python@3.11"]}"#,
    )?;

    let managers = detect_managers(temp_path, &[ToolType::Environments]);

    // Should detect both managers (the order may vary)
    assert_eq!(managers.len(), 2, "Should detect both mise and devbox");
    let manager_names: Vec<&str> = managers.iter().map(|(m, _)| m.name()).collect();
    assert!(manager_names.contains(&"mise"), "Should detect mise");
    assert!(manager_names.contains(&"devbox"), "Should detect devbox");

    Ok(())
}
