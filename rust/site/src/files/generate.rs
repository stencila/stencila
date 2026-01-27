//! Files index generation
//!
//! Generates a files index for sites with uploads enabled.
//! The index is organized by directory, with one JSON file per directory.

use std::collections::BTreeMap;
use std::path::Path;

use eyre::Result;
use tokio::fs;

use super::entry::FileEntry;

/// Statistics from files index generation
#[derive(Debug, Clone, Default)]
pub struct FilesIndexStats {
    /// Total number of files indexed
    pub total_files: usize,
    /// Number of directory index files written
    pub directories: usize,
}

/// Generate files index for a site
///
/// Creates `_files/` directory structure with JSON files for each directory
/// containing file metadata.
///
/// # Arguments
/// * `workspace_dir` - The workspace root directory (paths are relative to this)
/// * `site_root` - The root directory of the site source files
/// * `output_dir` - The output directory where the site was rendered
/// * `extensions` - Optional list of file extensions to include (case-insensitive, without leading dot)
pub async fn generate_files_index(
    workspace_dir: &Path,
    site_root: &Path,
    output_dir: &Path,
    extensions: Option<&[String]>,
) -> Result<FilesIndexStats> {
    let mut entries_by_dir: BTreeMap<String, Vec<FileEntry>> = BTreeMap::new();

    // Collect files recursively, with paths relative to workspace_dir
    collect_files_recursive(site_root, workspace_dir, extensions, &mut entries_by_dir).await?;

    // Write index files
    let files_dir = output_dir.join("_files");
    let mut stats = FilesIndexStats::default();

    for (dir, mut entries) in entries_by_dir {
        if entries.is_empty() {
            continue;
        }

        // Sort entries by path for deterministic output
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        stats.total_files += entries.len();
        stats.directories += 1;

        // Determine output path
        let index_path = if dir.is_empty() {
            files_dir.join("_root.json")
        } else {
            files_dir.join(format!("{dir}.json"))
        };

        // Create parent directories
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write JSON
        let json = serde_json::to_string(&entries)?;
        fs::write(&index_path, json).await?;
    }

    Ok(stats)
}

/// Recursively collect files from a directory
///
/// # Arguments
/// * `current_dir` - The current directory being scanned
/// * `workspace_dir` - The workspace root (paths are computed relative to this)
/// * `extensions` - Optional extension filter
/// * `entries_by_dir` - Map to collect entries grouped by parent directory
async fn collect_files_recursive(
    current_dir: &Path,
    workspace_dir: &Path,
    extensions: Option<&[String]>,
    entries_by_dir: &mut BTreeMap<String, Vec<FileEntry>>,
) -> Result<()> {
    let mut dir_entries = match fs::read_dir(current_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to read directory {}: {}", current_dir.display(), e);
            return Ok(());
        }
    };

    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden files and directories
        if file_name_str.starts_with('.') {
            continue;
        }

        // Skip files starting with underscore (e.g., _nav.yaml)
        if file_name_str.starts_with('_') {
            continue;
        }

        // Skip common non-content directories
        if file_name_str == "node_modules" || file_name_str == "__pycache__" {
            continue;
        }

        // Get file type without following symlinks to detect symlinks
        let file_type = match entry.file_type().await {
            Ok(ft) => ft,
            Err(e) => {
                tracing::warn!("Failed to read file type for {}: {}", path.display(), e);
                continue;
            }
        };

        // Skip symlinks to prevent loops and unexpected paths
        if file_type.is_symlink() {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to read metadata for {}: {}", path.display(), e);
                continue;
            }
        };

        if metadata.is_dir() {
            // Recurse into subdirectory
            Box::pin(collect_files_recursive(
                &path,
                workspace_dir,
                extensions,
                entries_by_dir,
            ))
            .await?;
        } else if metadata.is_file() {
            // Get file extension (final extension only, e.g., "file.tar.gz" -> "gz")
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            // Check extension filter (case-insensitive, strip leading dots from config)
            if let Some(allowed) = extensions {
                let ext_lower = extension.to_lowercase();
                if !allowed.iter().any(|a| {
                    let normalized = a.strip_prefix('.').unwrap_or(a);
                    normalized.to_lowercase() == ext_lower
                }) {
                    continue;
                }
            }

            // Skip files without extensions if we have an extension filter
            if extensions.is_some() && extension.is_empty() {
                continue;
            }

            // Get relative path from workspace root (matches document path convention)
            let rel_path = match path.strip_prefix(workspace_dir) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            // Get parent directory for grouping (also relative to workspace)
            let parent_dir = path
                .parent()
                .and_then(|p| p.strip_prefix(workspace_dir).ok())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();

            // Get last modified time
            let last_modified = metadata
                .modified()
                .map(|t| {
                    chrono::DateTime::<chrono::Utc>::from(t)
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string()
                })
                .unwrap_or_default();

            let file_entry = FileEntry::new(rel_path, metadata.len(), extension, last_modified);

            entries_by_dir
                .entry(parent_dir)
                .or_default()
                .push(file_entry);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    async fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<()> {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = File::create(&path).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_files_index() -> Result<()> {
        let workspace = TempDir::new()?;
        let output_dir = TempDir::new()?;

        // Create test files in site subdirectory (simulating root = "site")
        create_test_file(workspace.path(), "site/readme.md", "# Hello").await?;
        create_test_file(workspace.path(), "site/data/sample.csv", "a,b,c").await?;
        create_test_file(workspace.path(), "site/data/config.json", "{}").await?;
        create_test_file(workspace.path(), "site/docs/guide/intro.md", "# Intro").await?;

        let site_root = workspace.path().join("site");

        // Generate index (all files) - paths should be workspace-relative
        let stats =
            generate_files_index(workspace.path(), &site_root, output_dir.path(), None).await?;

        assert_eq!(stats.total_files, 4);
        assert_eq!(stats.directories, 3); // site, site/data, site/docs/guide

        // Verify site root index (not _root.json since site/ is the directory)
        let site_index = output_dir.path().join("_files/site.json");
        assert!(site_index.exists());
        let site_content: Vec<FileEntry> =
            serde_json::from_str(&fs::read_to_string(&site_index).await?)?;
        assert_eq!(site_content.len(), 1);
        assert_eq!(site_content[0].path, "site/readme.md");

        // Verify data index (now at site/data.json)
        let data_index = output_dir.path().join("_files/site/data.json");
        assert!(data_index.exists());
        let data_content: Vec<FileEntry> =
            serde_json::from_str(&fs::read_to_string(&data_index).await?)?;
        assert_eq!(data_content.len(), 2);
        // Paths should include site/ prefix
        assert!(
            data_content
                .iter()
                .any(|e| e.path == "site/data/sample.csv")
        );

        // Verify nested directory index
        let guide_index = output_dir.path().join("_files/site/docs/guide.json");
        assert!(guide_index.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_extension_filter() -> Result<()> {
        let workspace = TempDir::new()?;
        let output_dir = TempDir::new()?;

        // Create test files with different extensions
        create_test_file(workspace.path(), "data.csv", "a,b").await?;
        create_test_file(workspace.path(), "data.json", "{}").await?;
        create_test_file(workspace.path(), "readme.md", "# Hi").await?;

        // Generate index with extension filter (workspace and site_root are the same here)
        let extensions = vec!["csv".to_string(), "json".to_string()];
        let stats = generate_files_index(
            workspace.path(),
            workspace.path(),
            output_dir.path(),
            Some(&extensions),
        )
        .await?;

        assert_eq!(stats.total_files, 2); // Only csv and json

        let root_index = output_dir.path().join("_files/_root.json");
        let root_content: Vec<FileEntry> =
            serde_json::from_str(&fs::read_to_string(&root_index).await?)?;
        assert_eq!(root_content.len(), 2);

        // Verify md file is not included
        assert!(!root_content.iter().any(|e| e.extension == "md"));

        Ok(())
    }

    #[tokio::test]
    async fn test_extension_case_insensitive() -> Result<()> {
        let workspace = TempDir::new()?;
        let output_dir = TempDir::new()?;

        // Create file with uppercase extension
        create_test_file(workspace.path(), "DATA.CSV", "a,b").await?;

        // Filter with lowercase extension (workspace and site_root are the same here)
        let extensions = vec!["csv".to_string()];
        let stats = generate_files_index(
            workspace.path(),
            workspace.path(),
            output_dir.path(),
            Some(&extensions),
        )
        .await?;

        assert_eq!(stats.total_files, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_skips_underscore_prefixed_files() -> Result<()> {
        let workspace = TempDir::new()?;
        let output_dir = TempDir::new()?;

        // Create regular files and underscore-prefixed files
        create_test_file(workspace.path(), "readme.md", "# Hello").await?;
        create_test_file(workspace.path(), "_nav.yaml", "nav: []").await?;
        create_test_file(workspace.path(), "_config.json", "{}").await?;
        create_test_file(workspace.path(), "data/_hidden.csv", "a,b").await?;
        create_test_file(workspace.path(), "data/visible.csv", "c,d").await?;

        let stats =
            generate_files_index(workspace.path(), workspace.path(), output_dir.path(), None)
                .await?;

        // Only readme.md and data/visible.csv should be included
        assert_eq!(stats.total_files, 2);

        // Verify root index only contains readme.md
        let root_index = output_dir.path().join("_files/_root.json");
        let root_content: Vec<FileEntry> =
            serde_json::from_str(&fs::read_to_string(&root_index).await?)?;
        assert_eq!(root_content.len(), 1);
        assert_eq!(root_content[0].path, "readme.md");

        // Verify data index only contains visible.csv
        let data_index = output_dir.path().join("_files/data.json");
        let data_content: Vec<FileEntry> =
            serde_json::from_str(&fs::read_to_string(&data_index).await?)?;
        assert_eq!(data_content.len(), 1);
        assert_eq!(data_content[0].path, "data/visible.csv");

        Ok(())
    }
}
