use std::{
    io::Read,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use regex::Regex;
use stencila_node_media::embed_media;
use tokio::fs;

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Context, Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_biblio::decode::bibtex;
use stencila_codec_pandoc::{pandoc_from_format, root_from_pandoc};

use super::decode::arxiv_id_to_doi;

/// Decode an arXiv source file to a Stencila [`Node`]
#[tracing::instrument(skip(options))]
pub(super) async fn decode_arxiv_src(
    arxiv_id: &str,
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let bytes = fs::read(path).await?;

    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // Extraction directory next to the tar.gz file
    let extract_dir = path.with_extension("").with_extension(""); // Remove .tar.gz

    // arXiv serves either gzipped single files or gzipped tar files
    // Check if it's a tar file by looking for tar magic bytes
    let pandoc = if decompressed.len() > 262 && &decompressed[257..262] == b"ustar" {
        tracing::debug!("Detected tar archive, extracting to directory");

        // Create directory if it doesn't exist
        if !extract_dir.exists() {
            fs::create_dir_all(&extract_dir).await?;

            // Extract all files from tar
            let mut archive = tar::Archive::new(decompressed.as_slice());
            archive
                .unpack(&extract_dir)
                .wrap_err("Failed to extract tar archive")?;

            tracing::debug!("Extracted tar archive to {}", extract_dir.display());
        } else {
            tracing::debug!(
                "Using existing extracted directory: {}",
                extract_dir.display()
            );
        }

        // Find the main LaTeX file
        let main_tex_filename = find_main_tex_filename(&extract_dir).await?;

        // Change into extract_dir so that Pandoc flattens \include and \input properly
        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(&extract_dir)?;

        let result = pandoc_from_format(
            "",
            Some(&PathBuf::from(main_tex_filename)),
            "latex",
            &options,
        )
        .await;

        // Restore original working directory
        std::env::set_current_dir(current_dir)?;

        result?
    } else {
        // Single decompressed file, assume it's LaTeX
        tracing::debug!("Single file detected, creating temporary .tex file");

        let latex = String::from_utf8(decompressed).wrap_err("Failed to decode LaTeX as UTF-8")?;
        if latex.trim().is_empty() {
            bail!("Retrieved LaTeX content is empty");
        }

        pandoc_from_format(&latex, None, "latex", &options).await?
    };

    let (mut node, info) = root_from_pandoc(pandoc, Format::Latex, &options)?;

    if extract_dir.exists() {
        // Embed media
        embed_media(&mut node, Some(&extract_dir))?;

        // Find and decode any bibliography file and assign to references
        if let Ok(Some(bib_filename)) = find_bib_filename(&extract_dir).await {
            let bib_path = extract_dir.join(&bib_filename);
            if let Ok(bib_content) = fs::read_to_string(&bib_path).await {
                match bibtex(&bib_content) {
                    Ok(references) => {
                        if let Node::Article(article) = &mut node {
                            article.references = Some(references);
                            tracing::debug!("Added references from {bib_filename}",);
                        }
                    }
                    Err(error) => {
                        tracing::warn!(
                            "Failed to decode bibliography file {bib_filename}: {error}"
                        );
                    }
                }
            }
        }
    }

    // Set doi, and other metadata
    if let Node::Article(article) = &mut node {
        article.doi = Some(arxiv_id_to_doi(arxiv_id));
        article.options.repository = Some("https://arxiv.org".into());
        article.options.path = Some(["src/", arxiv_id].concat());
    }

    Ok((node, info))
}

/// Find the main LaTeX file in a directory containing extracted arXiv sources
async fn find_main_tex_filename(dir: &Path) -> Result<String> {
    let mut candidates = Vec::new();

    // Read all .tex files
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "tex")
            && let Some(filename) = path.file_name().and_then(|filename| filename.to_str())
        {
            candidates.push(filename.to_string());
        }
    }

    if candidates.is_empty() {
        bail!("No .tex files found in extracted archive");
    }

    // Compile regex once outside the loop
    let input_regex =
        Regex::new(r"\\(?:input|include)\s*\{[^}]+\}").expect("Failed to compile input regex");

    // Score each candidate file
    let mut best_candidate = None;
    let mut best_score = -1i32;

    for filename in candidates {
        let mut score = 0;

        // Try to read the file
        if let Ok(content) = fs::read_to_string(&dir.join(&filename)).await {
            // +10 points for having \documentclass
            if content.contains("\\documentclass") {
                score += 10;
            }

            // +1 point for each \input or \include (main files typically include others)
            score += input_regex.find_iter(&content).count() as i32;

            // Prefer common main file names
            let lower_name = filename.to_lowercase();
            if matches!(
                lower_name.as_str(),
                "main.tex" | "paper.tex" | "article.tex" | "ms.tex" | "manuscript.tex"
            ) {
                score += 5;
            }
        }

        if score > best_score {
            best_score = score;
            best_candidate = Some(filename);
        }
    }

    match best_candidate {
        Some(filename) => {
            tracing::debug!("Selected main file: {filename} (score: {best_score})",);
            Ok(filename)
        }
        None => bail!("No suitable main LaTeX file found"),
    }
}

/// Find a bibliography file in a directory containing extracted arXiv sources
async fn find_bib_filename(dir: &Path) -> Result<Option<String>> {
    let mut candidates = Vec::new();

    // Read all .bib files
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "bib")
            && let Some(filename) = path.file_name().and_then(|filename| filename.to_str())
        {
            candidates.push(filename.to_string());
        }
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    // Score each candidate file
    let mut best_candidate = None;
    let mut best_score = -1i32;

    for filename in candidates {
        let mut score = 0;

        // Prefer common bibliography file names
        let lower_name = filename.to_lowercase();
        score += match lower_name.as_str() {
            "references.bib" => 10,
            "refs.bib" => 9,
            "bibliography.bib" => 8,
            "library.bib" => 7,
            "main.bib" => 6,
            "bibl.bib" => 5,
            "bib.bib" => 4,
            _ => 0,
        };

        // Try to read the file and check if it looks like a valid bibliography
        if let Ok(content) = fs::read_to_string(&dir.join(&filename)).await {
            // +1 point for each @article, @book, @inproceedings etc.
            score += content.matches('@').count() as i32;
        }

        if score > best_score {
            best_score = score;
            best_candidate = Some(filename);
        }
    }

    match best_candidate {
        Some(filename) => {
            tracing::debug!("Selected bibliography file: {filename} (score: {best_score})");
            Ok(Some(filename))
        }
        None => Ok(None),
    }
}
