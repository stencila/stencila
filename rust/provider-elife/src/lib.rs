use provider::{Provider, ProviderTrait};

pub struct ElifeProvider {}

impl ProviderTrait for ElifeProvider {
    fn spec() -> Provider {
        Provider::new("elife")
    }
}
