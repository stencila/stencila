use provider::{Provider, ProviderTrait};

/// A provider for eLife
struct ElifeProvider {}

impl ProviderTrait for ElifeProvider {
    fn spec() -> Provider {
        Provider {}
    }
}
