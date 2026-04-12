use std::path::PathBuf;

use serde::Deserialize;
use stencila_codec::{
    stencila_format::Format,
    stencila_schema::{
        Article, ArticleOptions, Datatable, DatatableOptions, Node, PropertyValueOrString,
        SoftwareSourceCode, SoftwareSourceCodeOptions,
    },
};

use super::types::{MinimalRepository, TextMatch};

/// Code search result item from GitHub search API
#[derive(Deserialize)]
pub struct CodeSearchItem {
    /// File name
    pub name: String,
    /// File path within the repository
    pub path: String,
    /// SHA hash of the file
    pub sha: String,
    /// API URL for the file
    pub url: String,
    /// Git URL for the file
    pub git_url: String,
    /// HTML URL for viewing the file on GitHub
    pub html_url: String,
    /// Repository containing the file
    pub repository: MinimalRepository,
    /// Search relevance score
    pub score: f64,
    /// File size in bytes
    pub file_size: Option<i64>,
    /// Programming language of the file
    pub language: Option<String>,
    /// Last modified timestamp
    pub last_modified_at: Option<String>,
    /// Line numbers where matches were found
    pub line_numbers: Option<Vec<String>>,
    /// Text match highlighting information
    pub text_matches: Option<Vec<TextMatch>>,
}

impl From<CodeSearchItem> for Node {
    fn from(code: CodeSearchItem) -> Self {
        // Determine the format from the file path
        let path = PathBuf::from(&code.path);
        let format = Format::from_path(&path);

        let id = Some(code.html_url.clone());

        let repository = Some(code.repository.html_url);
        let path = Some(code.path);
        let commit = Some(code.sha);

        let url = Some(code.url);
        let identifiers = Some(vec![PropertyValueOrString::String(code.html_url.clone())]);

        match format {
            // For tabular data files, create a placeholder Datatable
            Format::Csv
            | Format::Tsv
            | Format::Parquet
            | Format::Arrow
            | Format::Xlsx
            | Format::Xls
            | Format::Ods => Node::Datatable(Datatable {
                id,
                options: Box::new(DatatableOptions {
                    url,
                    identifiers,
                    repository,
                    path,
                    commit,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            // For Jupyter notebooks and document formats, create an Article placeholder
            Format::Ipynb
            | Format::Markdown
            | Format::Myst
            | Format::Qmd
            | Format::Smd
            | Format::Latex => Node::Article(Article {
                id,
                options: Box::new(ArticleOptions {
                    url,
                    identifiers,
                    repository,
                    path,
                    commit,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            // For all other formats, use SoftwareSourceCode
            _ => Node::SoftwareSourceCode(SoftwareSourceCode {
                id,
                name: code.name,
                programming_language: code.language.unwrap_or_default(),
                path,
                repository,
                commit,
                options: Box::new(SoftwareSourceCodeOptions {
                    url,
                    identifiers,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }
    }
}
