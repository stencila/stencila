/// Download from a URL to a path
#[cfg(feature = "request")]
pub async fn download<P: AsRef<Path>>(url: &str, path: P) -> eyre::Result<()> {
    use std::{fs::File, io, path::Path};
    
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;
    Ok(())
}
