/// Download from a URL to a path
#[cfg(feature = "request")]
pub async fn download<P: AsRef<std::path::Path>>(url: &str, path: P) -> eyre::Result<()> {
    use std::{fs::File, io};

    // TODO: Allow an existing client to be passed into this function
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Stencila")
        .send()
        .await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(&path)?;
    io::copy(&mut bytes.as_ref(), &mut file)?;
    Ok(())
}
