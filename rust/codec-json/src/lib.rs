use std::{
    fs::{self, File},
    io::{Cursor, Read, Write},
    path::Path,
};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        serde_json::{Map, Value},
        zip::{self, write::FileOptions, ZipArchive},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};

pub mod r#trait;
use r#trait::JsonCodec as _;

/// The current version of Stencila
///
/// Used to include the version number for the `$schema` and `@content` URLs.
pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        from_path(path, options)
    }

    async fn from_bytes(
        &self,
        bytes: &[u8],
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        from_bytes(bytes, options)
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        from_str(str)
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        to_path(node, path, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        to_string(node, options)
    }
}

/**
 * Decode a node from a JSON or JSON+zip file
 */
pub fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    if !path.exists() {
        bail!("Path `{}` does not exist", path.display());
    }

    let mut options = options.unwrap_or_default();
    if options.format.is_none() {
        let format = Format::from_path(path);
        options.format = Some(format);
    }

    let mut file = fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    from_bytes(&content, Some(options))
}

/**
 * Decode a node from JSON or JSON+zip bytes
 */
pub fn from_bytes(bytes: &[u8], options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
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

    from_str(&string)
}

/**
 * Decode a node from a JSON string
 */
pub fn from_str(str: &str) -> Result<(Node, DecodeInfo)> {
    let node = Node::from_json(str)?;

    Ok((node, DecodeInfo::none()))
}

/**
 * Encode a node to a JSON or JSON+zip file
 */
pub fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<EncodeInfo> {
    // Implement `to_path, rather than `to_bytes`, so that, if encoding to `json.zip`,
    // the single file in the Zip archive can have the name minus `.zip`

    let mut options = options.unwrap_or_default();
    options.standalone = Some(true);
    let options = Some(options);

    let (string, ..) = to_string(node, options.clone())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
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
        fs::write(path, string)?;
    };

    Ok(EncodeInfo::none())
}

/**
 * Encode a node to a JSON string
 */
pub fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
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
            Value::String(format!(
                "https://stencila.org/v{STENCILA_VERSION}/{type}.schema.json"
            )),
        );
        root.insert(
            String::from("@context"),
            Value::String(format!(
                "https://stencila.org/v{STENCILA_VERSION}/context.jsonld"
            )),
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
