use binary::{async_trait::async_trait, binary_clone_box, Binary, BinaryTrait};

pub struct ChromiumBinary;

#[async_trait]
impl BinaryTrait for ChromiumBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "chromium",
            &[],
            &[
                "/Applications/Chromium.app/Contents/MacOS",
                "C:\\Program Files\\Chromium\\Application",
            ],
            &[],
        )
    }

    binary_clone_box!();
}
