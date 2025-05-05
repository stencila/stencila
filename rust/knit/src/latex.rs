use std::{
    fs::{File, create_dir_all, read_to_string, rename, write},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use rand::{Rng, distr::Alphanumeric, rng};

use codec_latex_trait::to_latex;
use common::{
    eyre::{Result, bail},
    once_cell::sync::Lazy,
    regex::Regex,
    tempfile::tempdir,
    tracing,
};
use format::Format;
use schema::{Article, Block, CodeChunk, Node, RawBlock, Section};

/// Decode a LaTeX to a knitted [`Article`]
#[tracing::instrument]
pub(super) fn latex_to_article(path: &Path) -> Result<Article> {
    tracing::trace!("Decoding LaTeX to Article");

    let latex = read_to_string(path)?;
    Ok(Article::new(latex_to_blocks(&latex)))
}

/// Encode a knitted [`Article`] to LaTeX
#[tracing::instrument(skip(article))]
pub(super) fn article_to_latex(article: &Article, path: &Path) -> Result<()> {
    tracing::trace!("Encoding Article to LaTeX");

    let temp = tempdir()?;

    let latex = blocks_to_latex(&article.content, &Format::Latex, temp.path())?;

    Ok(write(path, latex)?)
}

/// Encode a knitted [`Article`] to PDF
#[tracing::instrument(skip(article))]
pub(super) fn article_to_pdf(
    article: &Article,
    path: &Path,
    passthrough_args: &[String],
) -> Result<()> {
    tracing::trace!("Encoding Article to PDF");

    let temp = tempdir()?;

    let latex = blocks_to_latex(&article.content, &Format::Pdf, temp.path())?;

    let input = temp.path().join("main.tex");
    let output = temp.path().join("main.pdf");

    write(&input, latex)?;

    let status = Command::new("latex")
        .current_dir(temp.path())
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-output-format=pdf",
            "main.tex",
        ])
        .args(passthrough_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        let log = read_to_string(temp.path().join("main.log"))?;
        bail!("latex failed with log:\n\n{log}");
    }

    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }
    rename(&output, path)?;

    Ok(())
}

/// Encode a knitted [`Article`] to DOCX
#[tracing::instrument(skip(article))]
pub(super) fn article_to_docx(
    article: &Article,
    path: &Path,
    passthrough_args: &[String],
) -> Result<()> {
    tracing::trace!("Encoding Article to DOCX");

    let temp = tempdir()?;
    let temp_path = temp.path();

    // Uncomment the following path during development for debugging intermediate files
    // let temp_path = &PathBuf::from("temp");

    let latex = blocks_to_latex(&article.content, &Format::Docx, temp_path)?;

    let input = temp_path.join("main.tex");
    write(&input, latex)?;

    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }

    let status = Command::new("pandoc")
        .args([
            &input.to_string_lossy().to_string(),
            "-o",
            &path.to_string_lossy().to_string(),
        ])
        .args(passthrough_args)
        .stdout(Stdio::null())
        .status()?;
    if !status.success() {
        bail!("pandoc failed");
    }

    Ok(())
}

/// Decode LaTeX into a vector of [`Block`]s
fn latex_to_blocks(latex: &str) -> Vec<Block> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?sx)                                   # s = . matches \n, x = ignore whitespace & allow comments
            \\code\{(?P<code_cmd>[^}]*)\}                # \code{<...>}
          | \\begin\{code\} \s*                          # \begin{code}
              (?:\[(?P<code_opts>[^\]]*?)\])? \s*        #   [opt1, opt2]   ← OPTIONAL
              (?P<code_env>.*?)                          #   body (lazy)
            \\end\{code\}                                # \end{code}
          | \\begin\{island\} (?P<island>.*?) \\end\{island\}  # island env
        ",
        )
        .expect("invalid regex")
    });

    let mut blocks = Vec::new();
    let mut cursor = 0;

    for captures in RE.captures_iter(latex) {
        let m = captures.get(0).expect("always present");
        if m.start() > cursor {
            blocks.push(Block::RawBlock(RawBlock::new(
                Format::Latex.to_string(),
                latex[cursor..m.start()].into(),
            )));
        }

        if let Some(mat) = captures.name("code_cmd").or(captures.name("code_env")) {
            let code = mat.as_str().into();

            let mut programming_language = None;
            let mut is_echoed = None;
            let mut is_hidden = None;
            if let Some(options) = captures.name("code_opts") {
                for option in options
                    .as_str()
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    if option == "hide" {
                        is_hidden = Some(true);
                    } else if option == "echo" {
                        is_echoed = Some(true);
                    } else if programming_language.is_none() {
                        programming_language = Some(option.to_string());
                    }
                }
            }

            blocks.push(Block::CodeChunk(CodeChunk {
                programming_language,
                is_hidden,
                is_echoed,
                code,
                ..Default::default()
            }));
        } else if let Some(mat) = captures.name("island") {
            blocks.push(Block::Section(Section::new(latex_to_blocks(mat.as_str()))));
        }

        cursor = m.end();
    }

    if cursor < latex.len() {
        blocks.push(Block::RawBlock(RawBlock::new(
            Format::Latex.to_string(),
            latex[cursor..].into(),
        )));
    }

    blocks
}

/// Encode a vector of [`Block`]s to LaTeX
///
/// The `format` argument represents the final destination format. This function always
/// returns a LaTeX string but the style of that LaTeX is dependent upon the destination format.
fn blocks_to_latex(blocks: &Vec<Block>, format: &Format, dir: &Path) -> Result<String> {
    let mut latex = String::new();
    for block in blocks {
        match block {
            Block::RawBlock(RawBlock { content, .. }) => latex.push_str(&content),
            Block::CodeChunk(CodeChunk {
                code,
                outputs,
                is_hidden,
                ..
            }) => {
                if is_hidden.unwrap_or_default() {
                    continue;
                }

                let is_block = code.contains("\n");

                if let Some(outputs) = outputs {
                    let mut outputs_latex = String::new();
                    for output in outputs {
                        let output_latex = match output {
                            Node::ImageObject(image) => {
                                let path = if image.content_url.starts_with("data:") {
                                    images::data_uri_to_file(&image.content_url, dir)?
                                } else {
                                    image.content_url.clone()
                                };
                                highlight_png(dir, &PathBuf::from(&path))?;
                                [r"\includegraphics{", &path, "}"].concat()
                            }
                            _ => to_latex(output),
                        };
                        outputs_latex.push_str(&output_latex);
                    }

                    if matches!(format, Format::Docx) {
                        if !is_block {
                            let verbatim = [r"\verb!", &outputs_latex, "!"].concat();
                            latex.push_str(&verbatim)
                        } else if outputs_latex.contains("includegraphics") {
                            latex.push_str(&outputs_latex)
                        } else {
                            let png = latex_to_png(&outputs_latex, dir)?;
                            highlight_png(dir, &png)?;
                            let image =
                                [r"\includegraphics{", &png.to_string_lossy(), "}"].concat();
                            latex.push_str(&image)
                        }
                    } else {
                        latex.push_str(&outputs_latex);
                    }
                } else if matches!(format, Format::Latex) {
                    // If there are no outputs, and final format is LaTeX, then display code
                    if is_block {
                        latex.push_str(r"\begin{code}");
                        latex.push_str(&code);
                        latex.push_str(r"\end{code}");
                    } else {
                        latex.push_str(r"\code{");
                        latex.push_str(&code);
                        latex.push('}');
                    }
                }
            }
            Block::Section(Section { content, .. }) => {
                let section = blocks_to_latex(content, &Format::Latex, dir)?;
                let png = latex_to_png(&section, dir)?;
                highlight_png(dir, &png)?;
                let image = [
                    r"\includegraphics[width=\linewidth]{",
                    &png.to_string_lossy(),
                    "}",
                ]
                .concat();
                latex.push_str(&image)
            }
            _ => {}
        }
    }
    Ok(latex)
}

/// Compile the given LaTeX snippet to a PNG
///
/// Places the provided LaTeX within a standalone document and calls `latex`
/// and `dvipng` successively to generate a PNG from it.
#[tracing::instrument(skip(latex))]
fn latex_to_png(latex: &str, dir: &Path) -> Result<PathBuf> {
    tracing::trace!("Converting LaTex to PNG");

    // TODO: add more usepackages here and/or allow packages to be specified
    let latex = format!(
        r"
\documentclass[border=5pt,preview]{{standalone}}

\usepackage{{pdflscape}}

\begin{{document}}
{latex}
\end{{document}}
"
    );

    let id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let tex_name = format!("{id}.tex");
    let tex_path = dir.join(&tex_name);
    let dvi_name = format!("{id}.dvi");
    let png_name = format!("{id}.png");
    let png_path = dir.join(&png_name);

    let mut tex_file = File::create(&tex_path)?;
    tex_file.write_all(latex.as_bytes())?;

    let status = Command::new("latex")
        .current_dir(dir)
        .args(["-interaction=batchmode", &tex_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        let log = read_to_string(dir.join(format!("{id}.log")))?;
        bail!("latex failed with log:\n\n{log}");
    }

    let status = Command::new("dvipng")
        .current_dir(dir)
        .args(["-D300", "-o", &png_name, &dvi_name])
        .status()?;
    if !status.success() {
        bail!("dvipng failed");
    }

    Ok(png_path)
}

/// Highlight a PNG image a being a code output or an island
///
/// Currently, places a green border around the image. This could be
/// be made customizable in the future.
#[tracing::instrument]
fn highlight_png(dir: &Path, png: &Path) -> Result<()> {
    tracing::trace!("Highlighting PNG");

    const BORDER_COLOR: &str = "#32CD32";
    const BORDER_WIDTH: &str = "5";

    let status = Command::new("mogrify")
        .current_dir(dir)
        .args([
            "-bordercolor",
            BORDER_COLOR,
            "-border",
            BORDER_WIDTH,
            &png.to_string_lossy(),
        ])
        .status()?;
    if !status.success() {
        bail!("mogrify failed");
    }

    Ok(())
}
