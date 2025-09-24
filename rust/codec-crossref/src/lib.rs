use std::{sync::LazyLock, time::Duration};

use itertools::Itertools;
use monostate::MustBe;
use reqwest::Client;

use serde::Deserialize;
use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_biblio::decode::text_to_reference;
use stencila_codec_csl::Item;
use stencila_codec_doi::DoiCodec;
use stencila_codec_text::to_text;
use stencila_version::STENCILA_USER_AGENT;

/// A codec for searching the Crossref API for works
/// 
/// Primarily used for fetching the DOI and other metadata on a work.
pub struct CrossrefCodec;

#[async_trait]
impl Codec for CrossrefCodec {
    fn name(&self) -> &str {
        "crossref"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        StructuringOptions::none()
    }
}

impl CrossrefCodec {
    pub fn supports_identifier(_identifier: &str) -> bool {
        // Given that this codec searches the Crossref API this supports
        // any identifier
        true
    }

    #[tracing::instrument(skip(options))]
    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        static CLIENT: LazyLock<Client> = LazyLock::new(|| {
            Client::builder()
                .user_agent(STENCILA_USER_AGENT)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("invalid client")
        });

        const BASE_URL: &str = "https://api.crossref.org/works";

        let decode_info = DecodeInfo::none();
        let structuring_options = Self.structuring_options(&Format::Csl);

        // Decode the identifier into a reference
        let reference = text_to_reference(identifier);

        // If the reference has a DOI, then try to use that
        if let Some(doi) = reference.doi {
            tracing::debug!("Getting work by DOI");

            // Get work by DOI, allowing for failure
            let response = CLIENT.get(format!("{BASE_URL}/{doi}")).send().await?;
            if response.status().is_success()
                && let Ok(response) = response.json::<WorkResponse>().await {
                    let node = response.message.into();
                    return Ok((node, decode_info, structuring_options));
                };

            // If the above failed (e.g. DOI is a non-Crossref, DOI is invalid, network broke)
            // fallback try the DOI codec
            if let Ok(result) = DoiCodec::from_identifier(identifier, options).await {
                return Ok(result);
            }
        }

        // If the reference does not have a DOI, or the above failed, construct a search query
        // See https://api.crossref.org/swagger-ui/index.html#/Works/get_works
        let mut query = vec![];
        if let Some(title) = reference.title {
            // If the reference has a title (i.e. did not use Biblio codec's
            // fallback parser) then use that with year and/or authors
            query.push(("query.title", to_text(&title)));
            if let Some(year) = reference.date.and_then(|date| date.year()) {
                query.push(("query.biblio", year.to_string()));
            }
            if let Some(authors) = reference.authors {
                let authors = authors.into_iter().map(|author| author.name()).join(" ");
                query.push(("query.author", authors));
            }
        } else {
            // Otherwise, use the entire identifier as the free form "query" param
            query.push(("query", identifier.to_string()));
        }

        tracing::debug!("Searching CrossRef API");

        // Make the request, failing on error
        let response = CLIENT.get(BASE_URL).query(&query).send().await?;
        response.error_for_status_ref()?;
        let mut response: WorkListResponse = response.json().await?;

        if response.message.items.is_empty() {
            bail!("No works matched identifier");
        };
        let node = response.message.items.swap_remove(0).into();

        Ok((node, decode_info, structuring_options))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct WorkResponse {
    status: MustBe!("ok"),
    message_type: MustBe!("work"),
    pub message: Item,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct WorkListResponse {
    status: MustBe!("ok"),
    message_type: MustBe!("work-list"),
    pub message: WorkListMessage,
}

#[derive(Deserialize)]
pub struct WorkListMessage {
    pub items: Vec<Item>,
}
