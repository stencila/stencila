use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, NodeType,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        glob,
        tokio::{
            fs::{create_dir_all, read_to_string, remove_file, write},
            process::Command,
        },
        tracing,
    },
    format::Format,
    schema::Node,
    status::Status,
};
use codec_latex_trait::to_latex;
use rand::{Rng, distr::Alphanumeric, rng};

/// A codec for PNG
pub struct PngCodec;

#[async_trait]
impl Codec for PngCodec {
    fn name(&self) -> &str {
        "png"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Png => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::HighLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = options.unwrap_or_default();
        let tool = options.tool.clone().unwrap_or_default();

        let info = if tool == "latex" || tool.is_empty() {
            latex_to_png(node, path, &options).await?
        } else {
            bail!("Tool `{tool}` is not supported for encoding to PNG")
        };

        modify_png(path, options).await?;

        Ok(info)
    }
}

/// Encode a node to PNG using `latex` binary
#[tracing::instrument(skip(node))]
async fn latex_to_png(node: &Node, path: &Path, options: &EncodeOptions) -> Result<EncodeInfo> {
    tracing::trace!("Generating PNG using LaTeX");

    // Use a unique job name to be able to run `latex` in the current working directory
    // (because paths in \input and \includegraphics commands are relative to that)
    // whilst also being able to clean up temporary file afterwards
    let job: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Encode to string without `standalone` options
    let (mut latex, info) = to_latex(node, Format::Latex, false, true);

    //...and then wrap in standalone \documentclass if a \documentclass is not specified
    if !latex.contains("\\documentclass") {
        latex = [
            r"
\documentclass[border=5pt,preview]{standalone}

\usepackage{pdflscape}

\begin{document}

",
            &latex,
            r"
\end{document}
",
        ]
        .concat();
    }

    let input_file = format!("{job}.tex");
    write(&input_file, latex).await?;

    let latex_status = Command::new("latex")
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-output-format=dvi",
            "-jobname",
            &job,
            &input_file,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    let log_file = PathBuf::from(format!("{job}.log"));
    let log = if log_file.exists() {
        read_to_string(log_file).await?
    } else {
        String::new()
    };

    if let Some(dir) = path.parent() {
        create_dir_all(dir).await?;
    }
    let dvi_status = Command::new("dvipng")
        .args([
            "-D300",
            "-o",
            &path.to_string_lossy(),
            &format!("{job}.dvi"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    for path in glob::glob(&format!("{job}.*"))?.flatten() {
        remove_file(path).await?;
    }

    if !latex_status.success() {
        bail!("latex failed:\n\n{}", log);
    }
    if !dvi_status.success() {
        bail!("dvipng failed");
    }

    Ok(info)
}

/// Modify a generated PNG
#[tracing::instrument]
async fn modify_png(path: &Path, options: EncodeOptions) -> Result<()> {
    if options.tool_args.is_empty() {
        return Ok(());
    }

    tracing::trace!("Modifying PNG");

    let output = Command::new("mogrify")
        .args(options.tool_args)
        .arg(path)
        .output()
        .await?;
    if !output.status.success() {
        bail!(
            "mogrify failed:\n\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
