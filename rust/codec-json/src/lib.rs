use std::{
    fs::File,
    io::{Cursor, Read, Write},
    path::Path,
};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde_json::{Map, Value},
        tokio::{self, fs::create_dir_all},
        zip::{self, write::FileOptions, ZipArchive},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};

pub mod r#trait;
use r#trait::JsonCodec as _;

/// A codec for JSON
pub struct JsonCodec;

#[async_trait]
impl Codec for JsonCodec {
    fn name(&self) -> &str {
        "json"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Json | Format::JsonZip => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    fn supports_from_bytes(&self) -> bool {
        true
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Json | Format::JsonZip => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_bytes(
        &self,
        bytes: &[u8],
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let string = if let Some(Format::JsonZip) =
            options.as_ref().and_then(|options| options.format.as_ref())
        {
            let reader = Cursor::new(bytes);
            let mut zip = ZipArchive::new(reader)?;
            let mut file = zip.by_index(0)?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            contents
        } else {
            String::from_utf8(bytes.to_vec())?
        };

        self.from_str(&string, options).await
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        // Implement `to_path, rather than `to_bytes`, so that, if encoding to `json.zip`,
        // the single file in the Zip archive can have the name minus `.zip`

        let (string, ..) = self.to_string(node, options.clone()).await?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }

        if let Some(Format::JsonZip) = options.and_then(|options| options.format) {
            let zip_file = File::create(path)?;
            let mut zip = zip::ZipWriter::new(&zip_file);

            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.split('.').next())
                .unwrap_or("document");
            let filename = [filename, ".json"].concat();

            let options = FileOptions::default().unix_permissions(0o755);
            zip.start_file(filename, options)?;
            zip.write_all(string.as_bytes())?;
            zip.finish()?;
        } else {
            tokio::fs::write(path, string).await?;
        };

        Ok(EncodeInfo::none())
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let node = Node::from_json(str)?;

        Ok((node, DecodeInfo::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let EncodeOptions {
            standalone,
            compact,
            ..
        } = options.unwrap_or_default();

        if !standalone.unwrap_or_default() {
            return Ok((
                match compact {
                    Some(true) => node.to_json(),
                    Some(false) | None => node.to_json_pretty(),
                }?,
                EncodeInfo::none(),
            ));
        }

        let value = node.to_json_value()?;

        let value = if let (Some(true), Some(r#type)) = (
            standalone,
            value
                .as_object()
                .and_then(|object| object.get("type"))
                .and_then(|r#type| r#type.as_str())
                .map(String::from),
        ) {
            let object = value.as_object().expect("checked above").to_owned();

            // Insert the `$schema` and `@context` at the top of the root
            let mut root = Map::with_capacity(object.len() + 1);
            root.insert(
                String::from("$schema"),
                Value::String(format!("https://stencila.org/{type}.schema.json")),
            );
            root.insert(
                String::from("@context"),
                Value::String(String::from("https://stencila.org/context.jsonld")),
            );
            for (key, value) in object.into_iter() {
                root.insert(key, value);
            }

            Value::Object(root)
        } else {
            value
        };

        Ok((
            match compact {
                Some(true) => value.to_json(),
                Some(false) | None => value.to_json_pretty(),
            }?,
            EncodeInfo::none(),
        ))
    }
}
