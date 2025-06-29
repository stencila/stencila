use std::path::Path;

use codec_json::JsonCodec;
use codec_utils::reproducible_warnings;
use media_embed::embed_media;
use node_reconstitute::reconstitute;
use rust_embed::RustEmbed;

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{OptionExt, Result},
        serde_json,
        tempfile::tempdir,
        tokio::fs::write,
    },
    format::Format,
    schema::{Article, Node, Object, Primitive},
    status::Status,
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
};
use codec_pandoc::{
    coarse_to_path, pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc,
    root_to_pandoc,
};

mod decode;
mod encode;

#[cfg(test)]
mod tests;

/// A codec for Microsoft Word DOCX
pub struct DocxCodec;

const PANDOC_FORMAT: &str = "docx";
const DEFAULT_TEMPLATE: &str = "default.docx";

#[async_trait]
impl Codec for DocxCodec {
    fn name(&self) -> &str {
        "docx"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Docx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Docx => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let mut options = options.unwrap_or_default();

        // Default to extracting media to a temporary directory so that
        // they can be inlined into the decoded document
        let media_dir = tempdir()?;
        if !options.tool_args.join(" ").contains("--extract-media") {
            options
                .tool_args
                .push(format!("--extract-media={}", media_dir.path().display()));
        }

        let format = options.format.clone().unwrap_or(Format::Docx);
        let cache = options.cache.clone();

        let options = Some(options);
        let pandoc = pandoc_from_format("", Some(path), PANDOC_FORMAT, &options).await?;
        let (mut node, info) = root_from_pandoc(pandoc, format, &options)?;

        // Embed any images
        embed_media(&mut node, media_dir.path())?;

        let (data, mut properties) = decode::data_and_properties(path)?;

        if let Node::Article(article) = &mut node {
            if let Some(Primitive::String(source)) = properties.shift_remove("source") {
                article.options.source = Some(source);
            }

            if let Some(Primitive::String(commit)) = properties.shift_remove("commit") {
                article.options.commit = Some(commit);
            }

            if !properties.is_empty() {
                article.options.extra = Some(Object(properties));
            }
        }

        // If a cache is specified, or embedded in the DOCX, then use it to reconstitute the node
        let cache = if let Some(cache) = cache {
            let (cache, ..) = JsonCodec.from_path(&cache, None).await?;
            Some(cache)
        } else if let Some(cache) = data.get("cache.json") {
            let (cache, ..) = JsonCodec.from_str(cache, None).await?;
            Some(cache)
        } else {
            None
        };
        reconstitute(&mut node, cache.clone());

        // If a unedited version of the document is embedded in the DOCX, then add it to the
        // decoding info
        let unedited = if let Some(unedited) = data.get("unedited.json") {
            let (mut unedited, ..) = JsonCodec.from_str(unedited, None).await?;
            reconstitute(&mut unedited, cache);
            Some(unedited)
        } else {
            None
        };

        Ok((node, unedited, info))
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();

        // Default to render
        if options.render.is_none() {
            options.render = Some(true);
        }

        // Default to reproducible
        if options.reproducible.is_none() {
            options.reproducible = Some(true)
        }

        let format = options.format.clone().unwrap_or(Format::Docx);

        let reproducible = options.reproducible.unwrap_or_default();
        if reproducible {
            if let Node::Article(Article { options, .. }) = &node {
                reproducible_warnings(&options.source, &options.commit)
            }
        }

        // Default to using builtin template by extracting it to cache
        if options.template.is_none() {
            use dirs::{get_app_dir, DirType};
            let template = get_app_dir(DirType::Templates, true)?.join(DEFAULT_TEMPLATE);
            if !template.exists() {
                let file =
                    Templates::get(DEFAULT_TEMPLATE).ok_or_eyre("template does not exist")?;
                write(&template, file.data).await?;
            }
            options.template = Some(template);
        }

        let info = 'to_path: {
            // If a "coarse" article then encode directly from that format
            if let Node::Article(article) = node {
                if article.is_coarse(&Format::Latex) {
                    break 'to_path coarse_to_path(
                        node,
                        Format::Latex,
                        Format::Docx,
                        path,
                        Some(options),
                    )
                    .await;
                }
            }

            // Delegate to Pandoc
            let options = Some(options);
            let (pandoc, info) = root_to_pandoc(node, format.clone(), &options)?;
            pandoc_to_format(
                &pandoc,
                Some(path),
                &[PANDOC_FORMAT, "+native_numbering+styles"].concat(),
                &options,
            )
            .await?;

            Ok(info)
        }?;

        // Add any custom data
        let mut data = Vec::new();
        if reproducible {
            // Store node as JSON so the document can be reconstituted
            let (cache, ..) = JsonCodec
                .to_string(
                    node,
                    Some(EncodeOptions {
                        // Standalone so that schema version is included
                        standalone: Some(true),
                        ..Default::default()
                    }),
                )
                .await?;
            data.push(("cache.json".into(), cache));

            // Decode the DOCX (before adding cache) so that have a snapshot
            // of unedited, but backed out version of root node for rebasing
            let (unedited, ..) = DocxCodec.from_path(path, None).await?;
            let (unedited, ..) = JsonCodec
                .to_string(
                    &unedited,
                    Some(EncodeOptions {
                        // Standalone so that schema version is included
                        standalone: Some(true),
                        ..Default::default()
                    }),
                )
                .await?;
            data.push(("unedited.json".into(), unedited));
        }

        // Collect custom properties
        let mut properties = Vec::new();
        if let Node::Article(article) = node {
            if let Some(source) = &article.options.source {
                properties.push(("source".into(), source.clone()));
            }

            if let Some(commit) = &article.options.commit {
                properties.push(("commit".into(), commit.clone()));
            }

            if let Some(extra) = &article.options.extra {
                for (name, value) in extra.iter() {
                    let name = name.as_str();
                    let value = match value {
                        Primitive::String(value) => value.to_string(),
                        _ => serde_json::to_string(value).unwrap_or_default(),
                    };
                    properties.push((name.into(), value));
                }
            }
        }

        encode::data_and_properties(path, data, properties)?;

        Ok(info)
    }
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/templates"]
struct Templates;
