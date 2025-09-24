use std::{sync::LazyLock, time::Duration};

use reqwest::{Client, header};

use stencila_codec::{
    Codec, async_trait,
    eyre::{Result, bail},
    stencila_schema::{Node, Reference},
};
use stencila_codec_csl::CslCodec;
use stencila_version::STENCILA_USER_AGENT;

/// A codec for decoding DOIs into Stencila [`Node`]
///
/// This codec is used for fetching metadata for an [`Node`] having
/// a DOI. It is used to supplement other codecs, such as `codec-arxiv`,
/// `codec-openrxiv`, and `codec-pmcoa` by providing standardized metadata
/// for properties such as authors and references, which may not be well
/// supported by those codecs.
///
/// CSL-JSON is used because it is most widely supported across registries
/// such as DataCite and Crossref.
pub struct DoiCodec;

#[async_trait]
impl Codec for DoiCodec {
    fn name(&self) -> &str {
        "doi"
    }
}

impl DoiCodec {
    #[tracing::instrument(skip(reference))]
    pub async fn from_reference(reference: &Reference) -> Result<Node> {
        let Some(doi) = &reference.doi else {
            bail!("Reference does not have a DOI")
        };

        static CLIENT: LazyLock<Client> = LazyLock::new(|| {
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::ACCEPT,
                header::HeaderValue::from_static("application/vnd.citationstyles.csl+json"),
            );

            Client::builder()
                .default_headers(headers)
                .user_agent(STENCILA_USER_AGENT)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("invalid client")
        });

        let response = CLIENT.get(format!("https://doi.org/{doi}")).send().await?;
        response.error_for_status_ref()?;

        let json = response.text().await?;

        let (node, ..) = CslCodec.from_str(&json, None).await?;

        Ok(node)
    }
}
