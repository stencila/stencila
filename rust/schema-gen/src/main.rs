use common::{eyre::Result, tokio};
use schema_gen::schemas::Schemas;

#[tokio::main]
async fn main() -> Result<()> {
    let mut schemas = Schemas::read().await?;
    schemas.check()?;
    schemas.extend()?;
    schemas.expand()?;
    schemas.rust().await?;

    Ok(())
}
