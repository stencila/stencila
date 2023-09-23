use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, LossDirection, Losses,
};
use codec_jats_trait::JatsCodec as _;

mod decode;

/// A codec for JATS
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Jats => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Prose Inlines
            Text | Emphasis | Strong | Strikeout | Subscript | Superscript | Underline | Insert => NoLoss,
            Delete => HighLoss,
            // Prose Blocks
            Paragraph => NoLoss,
            // Works,
            Article => LowLoss,
            _ => None,
        }
    }

    async fn from_str(&self, str: &str, options: Option<DecodeOptions>) -> Result<(Node, Losses)> {
        decode::decode(str, options)
    }

    fn supports_to_format(&self, format: Format) -> CodecSupport {
        match format {
            Format::Jats => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        use NodeType::*;
        match node_type {
            // Data
            String | Cord => NoLoss,
            Null | Boolean | Integer | UnsignedInteger | Number => LowLoss,
            // Prose Inlines
            Text | Emphasis | Strong | Strikeout | Subscript | Superscript | Underline | Insert => NoLoss,
            Link | Parameter | AudioObject | ImageObject | MediaObject => LowLoss,
            // Prose Blocks
            Heading | Paragraph | ThematicBreak => NoLoss,
            List | ListItem | Table | TableRow | TableCell => LowLoss,
            // Code
            CodeFragment | CodeBlock => NoLoss,
            CodeExpression | CodeChunk => LowLoss,
            // Math
            MathFragment | MathBlock => NoLoss,
            // Works,
            Article => LowLoss,
            // If not in the above lists then no support
            _ => None,
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        if !matches!(node, Node::Article(..)) {
            return Ok((
                String::new(),
                Losses::of_everything(LossDirection::Encode, node.to_string()),
            ));
        }

        let (mut jats, losses) = node.to_jats();
        if !compact {
            jats = indent(&jats);
        }

        Ok((jats, losses))
    }
}

/// Indent JATS
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(jats: &str) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(jats);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(error) => panic!(
                "Error at position {}: {:?}",
                reader.buffer_position(),
                error
            ),
        }
        .expect("Failed to parse XML");
    }

    std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string()
}
