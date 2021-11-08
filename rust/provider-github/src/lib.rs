use provider::{Provider, ProviderTrait};

/// A provider for GitHub
struct GitHubProvider {}

impl ProviderTrait for GitHubProvider {
    fn spec() -> Provider {
        Provider {}
    }
}
