use eyre::Result;
use regex::Regex;

pub fn parse(url: &str) -> Result<url::Url> {
    // Ensure that url is fully formed
    let url = if url.starts_with(':') {
        format!("http://127.0.0.1{}", url)
    } else {
        let re = Regex::new("https?|wss?").unwrap();
        match re.captures(&url) {
            Some(_) => url.to_string(),
            None => format!("http://{}", url),
        }
    };
    match url::Url::parse(&url) {
        Ok(url) => Ok(url),
        Err(error) => Err(error.into()),
    }
}
