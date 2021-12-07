use binary::{
    async_trait::async_trait,
    Binary, BinaryTrait,
};

pub struct ChromiumBinary {}

#[async_trait]
impl BinaryTrait for ChromiumBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "chromium",
            &["Chromium"],
            &[
                "/Applications/Chromium.app/Contents/MacOS",
                "C:/Program Files/Chromium/Application",
            ],
            &[],
        )
    }
}
