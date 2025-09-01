use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, async_trait, eyre::Result, format::Format,
    schema::Node, status::Status,
};

mod cff;

use cff::{CffType, CitationFile};

/// A codec for the Citation File Format (CFF)
///
/// Only supports decoding from CFF. Primarily used for fetching
/// metadata about creative works hosted on GitHub.
///
/// CFF is a YAML-based format for providing citation metadata for software,
/// datasets, and other research outputs. Files are typically named CITATION.cff
/// and placed in the root of a repository.
///
/// See:
/// - https://citation-file-format.github.io/
/// - https://github.com/citation-file-format/citation-file-format
/// - https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-citation-files
pub struct CffCodec;

#[async_trait]
impl Codec for CffCodec {
    fn name(&self) -> &str {
        "cff"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cff => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let cff: CitationFile = serde_yaml::from_str(str)?;

        let node = if let Some(preferred_citation) = cff.preferred_citation {
            // If there's a preferred citation, convert that to an Article
            Node::Article(preferred_citation.into())
        } else {
            // Otherwise convert the CFF file itself based on its type
            match cff.work_type.as_ref().unwrap_or(&CffType::Software) {
                CffType::Software => {
                    // Decide between SoftwareSourceCode and SoftwareApplication
                    // based on available metadata
                    if cff.repository_code.is_some() || cff.repository.is_some() {
                        Node::SoftwareSourceCode(cff.into())
                    } else {
                        Node::SoftwareApplication(cff.into())
                    }
                }
                CffType::Dataset => {
                    // For now, treat datasets as software source code
                    // TODO: Add proper Dataset type to Stencila schema
                    Node::SoftwareSourceCode(cff.into())
                }
            }
        };

        Ok((node, DecodeInfo::default()))
    }
}
