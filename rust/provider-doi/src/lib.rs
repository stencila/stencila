use codec_csl::CslCodec;
use provider::{
    async_trait::async_trait,
    eyre::{bail, Result},
    http_utils::{get_json_with, headers},
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::*,
    Provider, ProviderParsing, ProviderTrait,
};

/// A provider that identifies and enriches `Article` and other `CreativeWork` nodes
///
/// Uses the DOI content negotiation protocol to fetch schema.org JSON-LD or CSL JSON
/// which is then decoded to Stencila node types.
#[derive(Debug, Default)]
pub struct DoiProvider {}

const DOI_URL: &str = "https://doi.org/";

#[async_trait]
impl ProviderTrait for DoiProvider {
    fn spec() -> Provider {
        Provider::new("doi")
    }

    /// Parse a [`CreativeWork`] nodes from a string
    ///
    /// Examples of DOI strings that this methods attempts to parse:
    ///
    /// - 10.5334/jors.182
    /// - DOI: 10.5334/jors.182
    /// - http://doi.org/10.5334/jors.182
    /// - https://doi.org/10.5334/jors.182
    ///
    /// See https://www.crossref.org/blog/dois-and-matching-regular-expressions/
    /// for notes on DOI matching.
    fn parse(string: &str) -> Vec<ProviderParsing> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b(10.\d{4,9}/[-._;()/:a-zA-Z0-9]+)\b").expect("Unable to create regex")
        });

        REGEX
            .captures_iter(string)
            .into_iter()
            .map(|captures| {
                let capture = captures.get(0).unwrap();

                let begin = capture.start();
                let end = capture.end();
                let node = Node::CreativeWork(CreativeWork {
                    identifiers: Some(vec![ThingIdentifiers::String(
                        [DOI_URL, capture.as_str()].concat(),
                    )]),
                    ..Default::default()
                });

                ProviderParsing { begin, end, node }
            })
            .collect()
    }

    /// Enrich a [`CreativeWork`] node using it's DOI
    ///
    /// If the node is a `CreativeWork` type with a DOI, then uses DOI content negotiation
    /// protocol to fetch CSL JSON to enrich properties of the node.
    async fn enrich(node: Node) -> Result<Node> {
        let url = match &node {
            Node::CreativeWork(CreativeWork { identifiers, .. })
            | Node::Article(Article { identifiers, .. }) => {
                identifiers.iter().flatten().find_map(|id| match id {
                    ThingIdentifiers::String(id) => {
                        if id.starts_with(DOI_URL) {
                            Some(id.clone())
                        } else {
                            None
                        }
                    }
                    ThingIdentifiers::PropertyValue(PropertyValue { value, .. }) => match value {
                        PropertyValueValue::String(id) => {
                            if id.starts_with(DOI_URL) {
                                Some(id.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                })
            }
            _ => None,
        };

        let url = match url {
            Some(url) => url,
            None => return Ok(node),
        };

        let data = get_json_with(
            &url,
            &[(headers::ACCEPT, "application/vnd.citationstyles.csl+json")],
        )
        .await?;

        CslCodec::from_json(data)
    }
}

#[cfg(test)]
mod tests {
    use test_utils::assert_json_is;

    use super::*;

    #[tokio::test]
    async fn detect() -> Result<()> {
        // Test that the extracted DOI is correct
        for str in [
            "10.5334/jors.182",
            "   10.5334/jors.182  ",
            "DOI: 10.5334/jors.182",
            " doi:10.5334/jors.182 and more",
            "DOI: http://doi.org/10.5334/jors.182",
            "https://doi.org/10.5334/jors.182",
        ] {
            let detections = DoiProvider::detect(&Node::String(str.to_string())).await?;
            match &detections[0].node {
                Node::CreativeWork(CreativeWork { identifiers, .. }) => {
                    assert_json_is!(identifiers, ["https://doi.org/10.5334/jors.182"])
                }
                _ => bail!("Unexpected return type"),
            };
        }

        // Test that a collection of DOI's as entered "in the wild" are detected
        for str in [
            "10.1016/j.celrep.2013.10.048",
            "10.1016/j.stem.2013.04.008",
            "10.1038/nature08320",
            "10.1038/nbt.2249",
            "10.1038/s41587-019-0035-0",
            "10.1038/sdata.2018.13",
            "10.1101/gad.1616208",
            "10.1101/gr.223925.117",
            "10.1242/dev.156844",
            "10.3791/51609",
            "10.5281/zenodo.5137374",
            "DOI: 10.1002/0471142301.ns0321s44",
            "DOI: 10.1002/dad2.12156",
            "DOI: 10.1002/mds.28621",
            "DOI: 10.1007/978-1-62703-574-3_15",
            "DOI: 10.1016/j.nbd.2021.105482",
            "DOI: 10.1038/s41598-021-93105-y",
            "doi: 10.1093/bioinformatics/btab385",
            "DOI: 10.1126/sciadv.abg4922      ",
            "DOI: 10.1186/s12859-021-04307-0",
            "doi: 10.18129/B9.bioc.dasper",
            "doi: https://doi.org/10.1007/s00401-021-02343-x",
            "doi: https://doi.org/10.1016/j.isci.2021.102484 (pre-proof)",
            "doi: https://doi.org/10.1101/2021.07.11.451956",
            "doi:10.1038/s41467-021-22108-0",
            "doi.org/10.1101/2021.03.02.433576",
            "dx.doi.org/10.17504/protocols.io.bnhtmb6n",
            "dx.doi.org/10.17504/protocols.io.bxrjpm4n",
            "http://doi.org/10.5281/zenodo.5012149",
            "http://dx.doi.org/10.21769/BioProtoc.3888",
            "https://doi.org/10.1002/mds.28755",
            "https://doi.org/10.1016/j.stem.2018.09.009",
            "https://doi.org/10.1038/s41467-021-22108-0",
            "https://doi.org/10.1073/pnas.2025053118",
            "https://doi.org/10.1083/jcb.202010004",
            "https://doi.org/10.15454/FLANUP",
            "https://doi.org/10.21203/rs.3.rs-220057/v2",
            "https://doi.org/10.3390/ijms22052689",
            "https://doi.org/10.5281/zenodo.5011869",
            "https://dx.doi.org/10.17504/protocols.io.bazhif36",
        ] {
            let detections = DoiProvider::detect(&Node::String(str.to_string())).await?;
            assert_eq!(detections.len(), 1)
        }

        Ok(())
    }
}
