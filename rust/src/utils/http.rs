use eyre::{Result};
use std::{fs::File, io, path::Path};

/// Download from a URL to a path
pub async fn download<P: AsRef<Path>>(url: &str, path: P) -> Result<()> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;
    Ok(())
}
