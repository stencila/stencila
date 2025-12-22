//! Initialize a workspace with stencila.toml configuration.
//!
//! This module provides the `stencila init` command for setting up
//! workspace configuration based on repository analysis.

use std::path::{Path, PathBuf};

use eyre::Result;
use glob::glob;

/// Detected project type(s).
///
/// A repository can have multiple project types detected simultaneously
/// (e.g., Python + Jupyter + Stencila content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    /// R project (.Rproj, .R files, renv/)
    RProject,
    /// Python project (pyproject.toml, requirements.txt, .py)
    PythonProject,
    /// Jupyter notebooks (.ipynb files)
    JupyterNotebook,
    /// Quarto project (_quarto.yml, .qmd files)
    QuartoProject,
    /// MyST Markdown project (myst.yml, _toc.yml, .myst files)
    MystProject,
    /// LaTeX project (.tex files)
    LatexProject,
    /// Static site (index.html, public/)
    StaticSite,
    /// Documentation site (docs/ with markdown)
    DocumentationSite,
    /// Stencila content (.smd files)
    StencilaContent,
    /// Node.js project (package.json)
    NodeProject,
    /// Empty or new project
    Empty,
}

/// Results of repository analysis.
#[derive(Debug, Default)]
pub struct RepoAnalysis {
    /// Detected project types (can be multiple)
    pub project_types: Vec<ProjectType>,
    /// Suggested site root directories in priority order
    pub suggested_roots: Vec<String>,
    /// Home page candidates in priority order
    pub home_page_candidates: Vec<PathBuf>,
    /// Executable document files (.smd, .qmd, .myst, .tex) for output suggestions
    pub executable_docs: Vec<PathBuf>,
}

/// Repository analyzer for detecting project characteristics.
pub struct RepoAnalyzer {
    base_dir: PathBuf,
}

impl RepoAnalyzer {
    /// Create a new analyzer for the given directory.
    pub fn new(base_dir: &Path) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
        }
    }

    /// Analyze the repository and return detected characteristics.
    pub fn analyze(&self) -> Result<RepoAnalysis> {
        let project_types = self.detect_project_types()?;
        let suggested_roots = self.detect_site_roots()?;
        let home_page_candidates = self.find_home_page_candidates(&suggested_roots)?;
        let executable_docs = self.find_executable_docs()?;

        Ok(RepoAnalysis {
            project_types,
            suggested_roots,
            home_page_candidates,
            executable_docs,
        })
    }

    /// Detect all applicable project types.
    fn detect_project_types(&self) -> Result<Vec<ProjectType>> {
        let mut types = Vec::new();

        // R Project detection
        if self.glob_exists("**/*.Rproj")?
            || (self.glob_exists("**/*.R")? && self.path_exists("renv"))
        {
            types.push(ProjectType::RProject);
        }

        // Python detection
        if self.path_exists("pyproject.toml")
            || self.path_exists("requirements.txt")
            || self.path_exists("setup.py")
            || self.glob_exists("**/*.py")?
        {
            types.push(ProjectType::PythonProject);
        }

        // Jupyter detection
        if self.glob_exists("**/*.ipynb")? {
            types.push(ProjectType::JupyterNotebook);
        }

        // Quarto detection
        if self.path_exists("_quarto.yml")
            || self.path_exists("_quarto.yaml")
            || self.glob_exists("**/*.qmd")?
        {
            types.push(ProjectType::QuartoProject);
        }

        // MyST detection
        if self.path_exists("myst.yml")
            || self.path_exists("_toc.yml")
            || self.glob_exists("**/*.myst")?
        {
            types.push(ProjectType::MystProject);
        }

        // LaTeX detection (check for .tex files with document structure)
        if self.glob_exists("**/*.tex")? {
            types.push(ProjectType::LatexProject);
        }

        // Node.js detection
        if self.path_exists("package.json") {
            types.push(ProjectType::NodeProject);
        }

        // Static site detection
        if self.path_exists("index.html") || self.path_exists("public/index.html") {
            types.push(ProjectType::StaticSite);
        }

        // Documentation site detection
        if self.path_exists("docs") && self.dir_has_content(&self.base_dir.join("docs"))? {
            types.push(ProjectType::DocumentationSite);
        }

        // Stencila content detection
        if self.glob_exists("**/*.smd")? {
            types.push(ProjectType::StencilaContent);
        }

        if types.is_empty() {
            types.push(ProjectType::Empty);
        }

        Ok(types)
    }

    /// Detect valid site root directories in priority order.
    fn detect_site_roots(&self) -> Result<Vec<String>> {
        // Candidates in priority order
        let candidates = ["site", "docs", "public", "content", "pages", "src"];

        let mut valid_roots = Vec::new();

        for candidate in candidates {
            let path = self.base_dir.join(candidate);
            if path.exists() && path.is_dir() && self.dir_has_content(&path)? {
                valid_roots.push(candidate.to_string());
            }
        }

        // Always include root as fallback
        valid_roots.push(".".to_string());

        Ok(valid_roots)
    }

    /// Find home page candidates within the suggested root directories.
    fn find_home_page_candidates(&self, roots: &[String]) -> Result<Vec<PathBuf>> {
        // Candidates in priority order
        let candidates = [
            "index.smd",
            "index.qmd",
            "index.myst",
            "index.md",
            "index.tex",
            "main.smd",
            "main.qmd",
            "main.tex",
            "main.md",
            "README.smd",
            "README.qmd",
            "README.md",
            "readme.md",
            "index.html",
        ];

        let mut found = Vec::new();

        // Check in the first suggested root (highest priority)
        let search_dir = roots
            .first()
            .map(|r| {
                if r == "." {
                    self.base_dir.clone()
                } else {
                    self.base_dir.join(r)
                }
            })
            .unwrap_or_else(|| self.base_dir.clone());

        for candidate in candidates {
            let path = search_dir.join(candidate);
            if path.exists() {
                // Store relative path from base_dir
                let relative = path
                    .strip_prefix(&self.base_dir)
                    .unwrap_or(&path)
                    .to_path_buf();
                found.push(relative);
            }
        }

        Ok(found)
    }

    /// Find all executable document files (.smd, .qmd, .myst, .tex).
    fn find_executable_docs(&self) -> Result<Vec<PathBuf>> {
        let mut docs = Vec::new();

        for pattern in ["**/*.smd", "**/*.qmd", "**/*.myst", "**/*.tex"] {
            docs.extend(self.glob_files(pattern)?);
        }

        // Sort for consistent ordering
        docs.sort();

        Ok(docs)
    }

    /// Suggest exclusion patterns filtered to only those relevant to the site root.
    ///
    /// Only includes patterns for file types/directories that actually exist
    /// within the site root directory.
    pub fn suggest_excludes_for_root(
        &self,
        project_types: &[ProjectType],
        site_root: &Path,
    ) -> Result<Vec<String>> {
        let mut excludes = vec![
            ".git/**".to_string(),
            ".stencila/**".to_string(),
            "stencila.local.toml".to_string(),
        ];

        // Helper to check if a pattern matches anything in site_root
        let pattern_exists = |pattern: &str| -> bool {
            let full_pattern = site_root.join(pattern).to_string_lossy().to_string();
            glob(&full_pattern)
                .map(|mut g| g.next().is_some())
                .unwrap_or(false)
        };

        // Helper to check if a directory exists in site_root
        let dir_exists = |dir: &str| -> bool { site_root.join(dir).is_dir() };

        for pt in project_types {
            match pt {
                ProjectType::PythonProject => {
                    if dir_exists("__pycache__") || pattern_exists("**/__pycache__") {
                        excludes.push("__pycache__/**".to_string());
                    }
                    if dir_exists(".venv") {
                        excludes.push(".venv/**".to_string());
                    }
                    if dir_exists("venv") {
                        excludes.push("venv/**".to_string());
                    }
                    if pattern_exists("**/*.pyc") {
                        excludes.push("*.pyc".to_string());
                    }
                    if dir_exists(".pytest_cache") {
                        excludes.push(".pytest_cache/**".to_string());
                    }
                    if pattern_exists("**/*.egg-info") {
                        excludes.push("*.egg-info/**".to_string());
                    }
                }
                ProjectType::RProject => {
                    if dir_exists("renv") {
                        excludes.push("renv/**".to_string());
                    }
                    if site_root.join(".Rhistory").exists() {
                        excludes.push(".Rhistory".to_string());
                    }
                    if site_root.join(".RData").exists() {
                        excludes.push(".RData".to_string());
                    }
                    if dir_exists(".Rproj.user") {
                        excludes.push(".Rproj.user/**".to_string());
                    }
                }
                ProjectType::QuartoProject => {
                    if dir_exists("_site") {
                        excludes.push("_site/**".to_string());
                    }
                    if dir_exists("_freeze") {
                        excludes.push("_freeze/**".to_string());
                    }
                    if dir_exists(".quarto") {
                        excludes.push(".quarto/**".to_string());
                    }
                    if pattern_exists("**/*_files") {
                        excludes.push("*_files/**".to_string());
                    }
                }
                ProjectType::MystProject => {
                    if dir_exists("_build") {
                        excludes.push("_build/**".to_string());
                    }
                    if dir_exists(".jupyter_cache") {
                        excludes.push(".jupyter_cache/**".to_string());
                    }
                }
                ProjectType::LatexProject => {
                    // LaTeX auxiliary files - only add if any .tex files exist
                    if pattern_exists("**/*.tex") {
                        // These are commonly generated and should be excluded
                        for ext in [
                            "aux", "log", "out", "toc", "lof", "lot", "bbl", "blg", "fls",
                        ] {
                            if pattern_exists(&format!("**/*.{ext}")) {
                                excludes.push(format!("*.{ext}"));
                            }
                        }
                        if pattern_exists("**/*.fdb_latexmk") {
                            excludes.push("*.fdb_latexmk".to_string());
                        }
                        if pattern_exists("**/*.synctex.gz") {
                            excludes.push("*.synctex.gz".to_string());
                        }
                        if pattern_exists("_minted-*") {
                            excludes.push("_minted-*/**".to_string());
                        }
                    }
                }
                ProjectType::NodeProject => {
                    if dir_exists("node_modules") {
                        excludes.push("node_modules/**".to_string());
                    }
                    if dir_exists("dist") {
                        excludes.push("dist/**".to_string());
                    }
                    if dir_exists("build") {
                        excludes.push("build/**".to_string());
                    }
                    if dir_exists(".next") {
                        excludes.push(".next/**".to_string());
                    }
                    if dir_exists(".nuxt") {
                        excludes.push(".nuxt/**".to_string());
                    }
                }
                ProjectType::JupyterNotebook => {
                    if dir_exists(".ipynb_checkpoints") {
                        excludes.push(".ipynb_checkpoints/**".to_string());
                    }
                }
                _ => {}
            }
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        excludes.retain(|x| seen.insert(x.clone()));

        Ok(excludes)
    }

    // Helper methods

    fn glob_exists(&self, pattern: &str) -> Result<bool> {
        let full_pattern = self.base_dir.join(pattern).to_string_lossy().to_string();
        Ok(glob(&full_pattern)?.next().is_some())
    }

    fn glob_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = self.base_dir.join(pattern).to_string_lossy().to_string();
        let paths: Vec<PathBuf> = glob(&full_pattern)?
            .filter_map(Result::ok)
            .filter_map(|p| p.strip_prefix(&self.base_dir).ok().map(PathBuf::from))
            .collect();
        Ok(paths)
    }

    fn path_exists(&self, path: &str) -> bool {
        self.base_dir.join(path).exists()
    }

    fn dir_has_content(&self, dir: &Path) -> Result<bool> {
        for pattern in [
            "*.md", "*.smd", "*.qmd", "*.myst", "*.tex", "*.html", "*.ipynb",
        ] {
            let p = dir.join(pattern).to_string_lossy().to_string();
            if glob(&p)?.next().is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_empty_repo() -> Result<()> {
        let dir = tempdir()?;
        let analyzer = RepoAnalyzer::new(dir.path());
        let analysis = analyzer.analyze()?;

        assert_eq!(analysis.project_types, vec![ProjectType::Empty]);
        assert!(analysis.suggested_roots.contains(&".".to_string()));

        Ok(())
    }

    #[test]
    fn test_python_project() -> Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("pyproject.toml"), "[project]")?;
        fs::write(dir.path().join("main.py"), "print('hello')")?;
        // Create __pycache__ directory to test exclusion detection
        fs::create_dir(dir.path().join("__pycache__"))?;

        let analyzer = RepoAnalyzer::new(dir.path());
        let analysis = analyzer.analyze()?;

        assert!(analysis.project_types.contains(&ProjectType::PythonProject));

        // Test that exclusion patterns are detected when directories exist
        let excludes = analyzer.suggest_excludes_for_root(&analysis.project_types, dir.path())?;
        assert!(excludes.contains(&"__pycache__/**".to_string()));

        Ok(())
    }

    #[test]
    fn test_stencila_content() -> Result<()> {
        let dir = tempdir()?;
        fs::create_dir(dir.path().join("docs"))?;
        fs::write(dir.path().join("docs/index.smd"), "# Hello")?;
        fs::write(dir.path().join("docs/report.smd"), "# Report")?;

        let analyzer = RepoAnalyzer::new(dir.path());
        let analysis = analyzer.analyze()?;

        assert!(
            analysis
                .project_types
                .contains(&ProjectType::StencilaContent)
        );
        assert!(
            analysis
                .project_types
                .contains(&ProjectType::DocumentationSite)
        );
        assert!(analysis.suggested_roots.contains(&"docs".to_string()));
        assert!(!analysis.executable_docs.is_empty());

        Ok(())
    }

    #[test]
    fn test_mixed_formats() -> Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("pyproject.toml"), "[project]")?;
        fs::write(dir.path().join("notebook.ipynb"), "{}")?;
        fs::write(dir.path().join("report.qmd"), "# Report")?;

        let analyzer = RepoAnalyzer::new(dir.path());
        let analysis = analyzer.analyze()?;

        assert!(analysis.project_types.contains(&ProjectType::PythonProject));
        assert!(
            analysis
                .project_types
                .contains(&ProjectType::JupyterNotebook)
        );
        assert!(analysis.project_types.contains(&ProjectType::QuartoProject));

        Ok(())
    }
}
