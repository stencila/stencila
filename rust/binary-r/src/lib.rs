use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};
use binary_asdf::AsdfBinary;

pub struct RBinary;

#[async_trait]
impl BinaryTrait for RBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "R",
            &[],
            &["C:\\Program Files\\R\\R-*\\bin"],
            &["4.1.0"],
        )
    }

    async fn install_version(&self, version: &str, os: &str, _arch: &str) -> Result<()> {
        if os == "linux" || os == "macos" {
            let asdf = AsdfBinary {}.require(None, true).await?;
            asdf.run(&["plugin", "add", "R"]).await?;
            asdf.run(&["install", "R", version]).await?;
            Ok(())
        } else {
            bail!("Installation of R on Windows is not yet supported")
        }
    }
}
