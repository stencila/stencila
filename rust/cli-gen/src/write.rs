//! Write command documentation files to disk

use std::path::{Path, PathBuf};

use crate::{extract::CommandDoc, markdown::generate_article};

/// Write all command documentation files recursively
pub async fn write_command_docs(
    dest: &Path,
    doc: &CommandDoc,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(file_path) = get_file_path(dest, &doc.path, doc.has_subcommands()) else {
        return Ok(());
    };

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Generate article and encode to markdown
    let article = generate_article(doc);
    stencila_codecs::to_path(&article, &file_path, None).await?;

    // Recursively write children
    for child in &doc.subcommands {
        Box::pin(write_command_docs(dest, child)).await?;
    }

    Ok(())
}

/// Determine the file path for a command's documentation
fn get_file_path(dest: &Path, path: &[String], has_children: bool) -> Option<PathBuf> {
    if path.len() == 1 {
        // Root command: site/docs/cli/index.md
        Some(dest.join("index.md"))
    } else if has_children {
        // Command with subcommands: site/docs/cli/site/index.md
        let dir: PathBuf = path[1..].iter().collect();
        Some(dest.join(dir).join("index.md"))
    } else {
        // Leaf command: site/docs/cli/site/push.md
        let dir: PathBuf = path[1..path.len() - 1].iter().collect();
        let name = path.last()?;
        let filename = format!("{name}.md");
        Some(dest.join(dir).join(filename))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_path_root() {
        let dest = Path::new("/site/docs/cli");
        let path = vec!["stencila".to_string()];
        let result = get_file_path(dest, &path, true);
        assert_eq!(result, Some(PathBuf::from("/site/docs/cli/index.md")));
    }

    #[test]
    fn test_get_file_path_with_children() {
        let dest = Path::new("/site/docs/cli");
        let path = vec!["stencila".to_string(), "config".to_string()];
        let result = get_file_path(dest, &path, true);
        assert_eq!(
            result,
            Some(PathBuf::from("/site/docs/cli/config/index.md"))
        );
    }

    #[test]
    fn test_get_file_path_leaf() {
        let dest = Path::new("/site/docs/cli");
        let path = vec![
            "stencila".to_string(),
            "config".to_string(),
            "get".to_string(),
        ];
        let result = get_file_path(dest, &path, false);
        assert_eq!(
            result,
            Some(PathBuf::from("/site/docs/cli/config/get.md"))
        );
    }

    #[test]
    fn test_get_file_path_deep_with_children() {
        let dest = Path::new("/site/docs/cli");
        let path = vec![
            "stencila".to_string(),
            "site".to_string(),
            "access".to_string(),
        ];
        let result = get_file_path(dest, &path, true);
        assert_eq!(
            result,
            Some(PathBuf::from("/site/docs/cli/site/access/index.md"))
        );
    }

    #[test]
    fn test_get_file_path_deep_leaf() {
        let dest = Path::new("/site/docs/cli");
        let path = vec![
            "stencila".to_string(),
            "site".to_string(),
            "access".to_string(),
            "public".to_string(),
        ];
        let result = get_file_path(dest, &path, false);
        assert_eq!(
            result,
            Some(PathBuf::from("/site/docs/cli/site/access/public.md"))
        );
    }
}
