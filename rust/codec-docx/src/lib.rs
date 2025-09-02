use std::{io::Write, path::Path};

use rust_embed::RustEmbed;
use tempfile::{NamedTempFile, tempdir};
use tokio::fs::{create_dir_all, write};

use stencila_codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType, async_trait,
    eyre::{OptionExt, Result},
    stencila_format::Format,
    stencila_schema::{Article, Node, Object, Primitive, strip_non_content},
    stencila_status::Status,
};
use stencila_codec_json::JsonCodec;
use stencila_codec_pandoc::{
    coarse_to_path, pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc,
    root_to_pandoc,
};
use stencila_media_embed::embed_media;
use stencila_node_reconstitute::reconstitute;
use stencila_version::STENCILA_VERSION;

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
            if let Some(Primitive::String(repository)) = properties.shift_remove("repository") {
                article.options.repository = Some(repository);
            }

            if let Some(Primitive::String(path)) = properties.shift_remove("path") {
                article.options.path = Some(path);
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

        let mut re_encoding = false;

        // Re-use any encoding options previously set and in `extra`s
        // This may be the case, for example, when saving a DOCX previously
        // rendered, and then saving it again after execution.
        if let Node::Article(Article {
            options: article_options,
            ..
        }) = node
            && let Some(Primitive::Object(previous_options)) = article_options
                .extra
                .as_ref()
                .and_then(|extra| extra.get("encoding"))
        {
            re_encoding = true;

            if options.render.is_none()
                && let Some(Primitive::Boolean(render)) = previous_options.get("render")
            {
                options.render = Some(*render);
            }
            if options.reproducible.is_none()
                && let Some(Primitive::Boolean(reproducible)) = previous_options.get("reproducible")
            {
                options.reproducible = Some(*reproducible);
            }
            if options.highlight.is_none()
                && let Some(Primitive::Boolean(highlight)) = previous_options.get("highlight")
            {
                options.highlight = Some(*highlight);
            }
        }

        // Default to render
        if options.render.is_none() {
            options.render = Some(true);
        }

        let reproducible = options.reproducible.unwrap_or_default();

        // Default to highlighting if reproducible
        if reproducible && options.highlight.is_none() {
            options.highlight = Some(true);
        }

        if options.template.is_none() {
            if re_encoding {
                // Use the document itself as the template so that any styling
                // changes are maintained
                options.template = Some(path.into());
            } else {
                // Default to using builtin template by extracting it to cache
                // The cache path includes the Stencila version so that it is cache-busted
                // for each new version
                use stencila_dirs::{DirType, get_app_dir};
                let template = get_app_dir(DirType::Templates, false)?
                    .join(STENCILA_VERSION)
                    .join(DEFAULT_TEMPLATE);
                if cfg!(debug_assertions) || !template.exists() {
                    if let Some(parent) = template.parent() {
                        create_dir_all(parent).await?;
                    }
                    let file =
                        Templates::get(DEFAULT_TEMPLATE).ok_or_eyre("template does not exist")?;
                    write(&template, file.data).await?;
                }
                options.template = Some(template);
            }
        }

        let info = 'to_path: {
            let format = options.format.clone().unwrap_or(Format::Docx);
            let options = Some(options.clone());

            // If a "coarse" article then encode directly from that format
            if let Node::Article(article) = node
                && article.is_coarse(&Format::Latex)
            {
                break 'to_path coarse_to_path(node, Format::Latex, Format::Docx, path, options)
                    .await;
            }

            // Delegate to Pandoc
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
            data.push(("cache.json".into(), cache.clone()));

            // Decode the DOCX so that we have a snapshot of the unedited (but reconstituted)
            // version of the root node for rebasing. The cache options require a path to the
            // cache (not the node itself) so write a temporary file.
            let mut temp_file = NamedTempFile::with_suffix("json")?;
            temp_file.write_all(cache.as_bytes())?;
            let (mut unedited, ..) = DocxCodec
                .from_path(
                    path,
                    Some(DecodeOptions {
                        cache: Some(temp_file.path().into()),
                        ..Default::default()
                    }),
                )
                .await?;

            // Do avoid unnecessary duplication of data, that will be in the cache, remove
            // non-content properties that are not needed when reconstituted
            strip_non_content(&mut unedited);

            // Embed the unedited file into the document
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
        let mut properties = vec![("generator".into(), format!("Stencila {STENCILA_VERSION}"))];

        // Store encoding options excluding those that are not used and which may
        // have local paths in them
        if let Ok(encoding) = serde_json::to_string(&EncodeOptions {
            format: options.format,
            render: options.render,
            reproducible: options.reproducible,
            highlight: options.highlight,
            ..Default::default()
        }) {
            properties.push(("encoding".into(), encoding));
        }

        if let Node::Article(article) = node {
            if let Some(repository) = &article.options.repository {
                properties.push(("repository".into(), repository.clone()));
            }

            if let Some(path) = &article.options.path {
                properties.push(("path".into(), path.clone()));
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
