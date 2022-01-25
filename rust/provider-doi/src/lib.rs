use node_pointer::{bail, walk, Address, Visitor};
use once_cell::sync::Lazy;
use provider::{
    async_trait, stencila_schema::*, Provider, ProviderDetection, ProviderTrait, Result,
};
use regex::Regex;

/// A provider that identifies and enriches `Article` and other `CreativeWork` nodes
///
/// Uses the DOI content negotiation protocol to fetch schema.org JSON-LD or CSL JSON
/// which is then decoded to Stencila node types.
#[derive(Debug, Default)]
pub struct DoiProvider {}

const PROVIDER_NAME: &str = "DOI";

const DOI_URL: &str = "https://doi.org/";

#[async_trait]
impl ProviderTrait for DoiProvider {
    fn spec(&self) -> Provider {
        Provider {
            name: PROVIDER_NAME.to_string(),
        }
    }

    /// Detect [`CreativeWork`] nodes in a root node by searching for DOIs
    ///
    /// Examples of DOI strings that this provider attempts to detect:
    ///
    /// - 10.5334/jors.182
    /// - DOI: 10.5334/jors.182
    /// - http://doi.org/10.5334/jors.182
    /// - https://doi.org/10.5334/jors.182
    ///
    /// See https://www.crossref.org/blog/dois-and-matching-regular-expressions/
    /// for notes on DOI matching.
    async fn detect(&self, root: &Node) -> Result<Vec<ProviderDetection>> {
        let mut detector = DoiDetector::default();
        walk(root, &mut detector);
        Ok(detector.detected)
    }

    /// Identify a [`CreativeWork`] node
    ///
    /// Currently just checks that the work has an identifier that is a DOI
    /// (`detect()` already identifies by extracting a DOI).
    async fn identify(&self, node: &Node) -> Result<Node> {
        match node {
            Node::CreativeWork(CreativeWork { identifiers, .. }) => {
                let found = identifiers.iter().flatten().any(|id| match id {
                    ThingIdentifiers::String(id) => id.starts_with(DOI_URL),
                    ThingIdentifiers::PropertyValue(PropertyValue { value, .. }) => match value {
                        PropertyValueValue::String(value) => value.starts_with(DOI_URL),
                        _ => false,
                    },
                });
                if found {
                    Ok(node.clone())
                } else {
                    bail!("Node does not appear to have a DOI")
                }
            }
            _ => bail!("Unexpected type of node"),
        }
    }
}

/// A [`Visitor`] that walks a node tree and detects DOIs representing
/// `CreativeWork` nodes.
#[derive(Debug, Default)]
pub struct DoiDetector {
    /// The detected nodes
    detected: Vec<ProviderDetection>,
}

impl Visitor for DoiDetector {
    /// Visit an inline node, and if it is a string, attempt to detect DOIs within it
    fn visit_inline(&mut self, address: &Address, node: &InlineContent) -> bool {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b(10.\d{4,9}/[-._;()/:a-zA-Z0-9]+)\b").expect("Unable to create regex")
        });

        if let InlineContent::String(content) = node {
            let mut detected = REGEX
                .captures_iter(content)
                .into_iter()
                .map(|captures| {
                    let capture = captures.get(0).unwrap();

                    let begin = address.add_index(capture.start());
                    let end = address.add_index(capture.end());
                    let node = Node::CreativeWork(CreativeWork {
                        identifiers: Some(vec![ThingIdentifiers::String(
                            ["https://doi.org/", capture.as_str()].concat(),
                        )]),
                        ..Default::default()
                    });

                    ProviderDetection {
                        provider: PROVIDER_NAME.to_string(),
                        begin,
                        end,
                        node,
                    }
                })
                .collect();
            self.detected.append(&mut detected);
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use node_pointer::bail;
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
            let detections = DoiProvider {}
                .detect(&Node::Paragraph(Paragraph {
                    content: vec![InlineContent::String(str.to_string())],
                    ..Default::default()
                }))
                .await?;

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
            let detections = DoiProvider {}
                .detect(&Node::Paragraph(Paragraph {
                    content: vec![InlineContent::String(str.to_string())],
                    ..Default::default()
                }))
                .await?;
            assert_eq!(detections.len(), 1)
        }

        Ok(())
    }
}
