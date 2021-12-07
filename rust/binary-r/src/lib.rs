use binary::{async_trait::async_trait, Binary, BinaryTrait};

pub struct RBinary {}

#[async_trait]
impl BinaryTrait for RBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "R",
            &["Rscript"],
            &["C:/Program Files/R/R-*/bin"],
            &[],
        )
    }
}
