use std::path::Path;

use stencila_codec::{
    EncodeInfo, EncodeOptions,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_latex_trait::to_latex;

use crate::format_to_path;

/// Encode a node that has been encoded with the `--coarse` option to a path
///
/// Rather than transforming to Pandoc JSON and then to encoding to the destination format
/// as usual, this transforms all nodes to the `from` format and then directly to the `to` format.
#[tracing::instrument(skip(node, options))]
pub async fn coarse_to_path(
    node: &Node,
    from: Format,
    to: Format,
    path: &Path,
    options: Option<EncodeOptions>,
) -> Result<EncodeInfo> {
    let options = options.unwrap_or_default();
    let standalone = options.standalone.unwrap_or(true);
    let render = options.render.unwrap_or(true);
    let highlight = options.highlight.unwrap_or(false);
    let reproducible = options.reproducible.unwrap_or(false);

    let (content, info) = match from {
        Format::Latex | Format::Tex => to_latex(
            node,
            to.clone(),
            standalone,
            render,
            highlight,
            reproducible,
        ),
        _ => bail!("Unsupported from format: {from}"),
    };

    format_to_path(&from, &to, &content, path, &Some(options)).await?;

    Ok(info)
}
