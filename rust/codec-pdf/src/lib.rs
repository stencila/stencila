use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use rand::{distr::Alphanumeric, rng, Rng};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        glob,
        tokio::{
            fs::{create_dir_all, read_to_string, remove_file, rename},
            process::Command,
        },
        tracing,
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, EncodeInfo, EncodeOptions, NodeType,
};
use codec_latex::LatexCodec;
use codec_pandoc::{pandoc_to_format, root_to_pandoc};

/// A codec for PDF
pub struct PdfCodec;

const PANDOC_FORMAT: &str = "pdf";

#[async_trait]
impl Codec for PdfCodec {
    fn name(&self) -> &str {
        "pdf"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pdf => CodecSupport::HighLoss,
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
        let tool = options
            .as_ref()
            .and_then(|opts| opts.tool.clone())
            .unwrap_or_default();

        if tool == "pandoc" {
            let (pandoc, info) = root_to_pandoc(node, Format::Pdf, &options)?;
            pandoc_to_format(&pandoc, Some(path), PANDOC_FORMAT, &options).await?;
            Ok(info)
        } else if tool.ends_with("latex") || tool.is_empty() {
            latex_to_pdf(node, path, options).await
        } else {
            bail!("Tool `{tool}` is not supported for encoding to PDF")
        }
    }
}

/// Encode a node to PDF using `latex` binary
#[tracing::instrument(skip(node))]
async fn latex_to_pdf(
    node: &Node,
    path: &Path,
    options: Option<EncodeOptions>,
) -> Result<EncodeInfo> {
    let options = options.unwrap_or_default();

    // Use a unique job name to be able to run `latex` in the current working directory
    // (because paths in \input and \includegraphics commands are relative to that)
    // whilst also being able to clean up temporary file afterwards
    let job: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let input_file = format!("{job}.tex");

    let info = LatexCodec
        .to_path(
            node,
            &PathBuf::from(&input_file),
            Some(EncodeOptions {
                standalone: Some(true),
                render: Some(true),
                // Indicate that the LaTeX should be generated for PDF as final
                // destination format
                format: Some(Format::Pdf),
                ..Default::default()
            }),
        )
        .await?;

    let tool = options
        .tool
        .clone()
        .unwrap_or_else(|| "xelatex".to_string());

    let status = Command::new(&tool)
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-jobname",
            &job,
            &input_file,
        ])
        .args(options.tool_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    let output_file = PathBuf::from(format!("{job}.pdf"));
    if output_file.exists() {
        if let Some(dir) = path.parent() {
            create_dir_all(dir).await?;
        }
        rename(output_file, path).await?;
    }

    let log_file = PathBuf::from(format!("{job}.log"));
    let log = if log_file.exists() {
        read_to_string(log_file).await?
    } else {
        String::new()
    };

    for path in glob::glob(&format!("{job}.*"))?.flatten() {
        remove_file(path).await?;
    }

    if !status.success() {
        bail!("{tool} failed:\n\n{}", log);
    }

    Ok(info)
}
