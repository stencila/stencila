use binary::{async_trait::async_trait, Binary, BinaryTrait};

pub struct RscriptBinary {}

#[async_trait]
impl BinaryTrait for RscriptBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "Rscript",
            &[],
            &["C:\\Program Files\\R\\R-*\\bin"],
            &[],
        )
    }
}
