use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use codec_lexical::LexicalCodec;
use jsonwebtoken as jwt;
use url::Host;

use codec::{Codec, EncodeOptions};
use common::{
    clap::{self, Parser},
    eyre::{bail, eyre, Context, Result},
    reqwest::{Client, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    tracing,
};
use document::{CommandWait, Document};
use schema::{Node, PropertyValueOrString};

const SECRET_NAME: &str = "GHOST_ADMIN_API_KEY";

/// Publish to Ghost
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file or directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Dry run test
    ///
    /// When set, stencila will perform the document conversion but skip the publication to Ghost.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// The Ghost domain
    ///
    /// This is the domain name of your Ghost instance, with an optional port.
    #[arg(long, env = "STENCILA_GHOST_DOMAIN", value_parser = parse_host)]
    ghost: Host,

    /// The Ghost Admin API key
    ///
    /// To create one, create a new Custom Integration under
    /// the Integrations screen in Ghost Admin. Use the Admin API Key,
    /// rather than the Content API Key.
    #[arg(long, env = "STENCILA_GHOST_KEY", value_parser = parse_key)]
    key: String,

    /// Create a page
    #[arg(long, conflicts_with = "post")]
    page: bool,

    /// Create a post
    #[arg(long, conflicts_with = "page")]
    post: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        if !self.path.exists() {
            bail!("Path does not exist: {}", self.path.display())
        }

        if !self.path.is_file() {
            bail!("Only publishing files is currently supported")
        }

        // Compile document
        let doc = Document::open(&self.path).await?;
        doc.compile(CommandWait::Yes).await?;

        // Get title, content and metadata from document
        // Title is required.
        let mut title = String::from("Untitled");
        let lexical = {
            let node = &*doc.root_read().await;

            if let schema::Node::Article(article) = node {
                if let Some(inlines) = &article.title {
                    title = codec_text::to_text(inlines);
                }
            }

            let (lexical, ..) = LexicalCodec
                .to_string(
                    node,
                    Some(EncodeOptions {
                        standalone: Some(false),
                        ..Default::default()
                    }),
                )
                .await?;

            lexical
        };

        // Generate JWT for request
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Construct URL for the REST resource
        // "root key" is the terminology use by Ghost/Lexical
        let root_key = if self.post {
            "posts"
        } else if self.page {
            "pages"
        } else {
            bail!("Please use --post or --page flag");
        };
        let resource_url = format!("https://{}/ghost/api/admin/{}/", self.ghost, root_key);

        // Construct JSON payload
        // See https://ghost.org/docs/admin-api/#creating-a-post
        // and https://github.com/stencila/stencila/issues/2481 for how they map to CLI args
        let payload = serde_json::json!({
            root_key : [
                json!({
                    "title": title,
                    "lexical": lexical,
                    "status": "draft",
                })
            ]
        });

        // Return early if this is just a dry run
        if self.dry_run {
            return Ok(());
        }

        // Send the request
        let response = Client::new()
            .post(&resource_url)
            .header("Authorization", format!("Ghost {token}"))
            .json(&payload)
            .send()
            .await?;

        // Handle the response...
        if let StatusCode::CREATED = response.status() {
            let Some(location) = response.headers().get("location") else {
                tracing::error!(resp = ?response, "POST succeeded, but Location header unavailable");
                bail!("Uploading the document to Ghost appears to have succeeded, but Ghost did not provide the new URL. Check Ghost Admin for the new draft.");
            };

            let location = location
                .to_str()
                .context("interpreting Location HTTP header")?;
            let ghost_id = location
                .strip_prefix(&resource_url)
                .map(|id| id.trim_start_matches('/').trim_end_matches('/'));

            if let Some(id) = ghost_id {
                {
                    tracing::debug!("Acquiring doc write lock");
                    let node: &mut Node = &mut *doc.root_write().await;

                    if let Node::Article(article) = node {
                        let identifier =
                            PropertyValueOrString::String(format!("{resource_url}{id}"));
                        match article.options.identifiers.as_mut() {
                            Some(ids) => ids.push(identifier),
                            None => article.options.identifiers = Some(vec![identifier]),
                        }
                    }
                }

                doc.save(CommandWait::Yes).await?;
            }

            Ok(())
        } else {
            tracing::error!(resp = ?response, "POST failed");

            let code = response.status().as_u16();

            if let Ok(body) = response.text().await {
                if let Ok(err) = serde_json::from_str::<serde_json::Value>(&body) {
                    bail!(
                        "HTTP {code}: {msg}:\n{err}",
                        msg = err["errors"][0]["message"]
                    )
                } else {
                    bail!("HTTP {code}: {body}")
                }
            } else {
                bail!("HTTP {code}s")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct Claims {
    // "Audience", e.g. a URL in the Ghost instance
    aud: String,
    iat: u64,
    exp: u64,
}

fn generate_jwt(key: &str) -> Result<String> {
    let Some((id, secret)) = key.split_once(':') else {
        return Err(eyre!("invalid Ghost Admin API key")); // should never happen because
    };

    let iat = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| eyre!("error accessing system clock: {}", e))?
        .as_secs();

    let exp = iat + 5 * 60; // 5 minutes
    let aud = "/admin/".to_string();

    let mut header = jwt::Header::new(jwt::Algorithm::HS256);
    header.typ = Some("JWT".to_string());
    header.kid = Some(id.to_string());

    let payload = Claims { aud, iat, exp };

    let secret: Result<Vec<u8>> = secret
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let hex_pair = std::str::from_utf8(chunk)?; // will always succeed as we start with a str

            u8::from_str_radix(hex_pair, 16).map_err(|e| eyre!("invalid input in secret: {}", e))
        })
        .collect();
    let secret = secret?;

    let secret = jwt::EncodingKey::from_secret(&secret);
    let token = jwt::encode(&header, &payload, &secret)?;

    Ok(token)
}

fn only_hex(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_lowercase() && c.is_ascii_hexdigit())
}

// Validate that key looks like a Ghost Admin API key
fn validate_key(key: &str) -> Result<String> {
    // Split into id:secret
    let Some((id, secret)) = key.split_once(':') else {
        bail!("Ghost Admin API key must be in format `id:secret`, i.e. an id and secret separated by a colon.");
    };

    if id.is_empty() {
        bail!("The id field of `key` must not be empty");
    }
    if secret.is_empty() {
        bail!("The secret field of `key` must not be empty");
    }

    if !only_hex(id) || !only_hex(secret) {
        tracing::warn!("Ghost Admin API key may be invalid; should only contain lowercase hexadecimal characters");
    }

    Ok(key.to_string())
}

/// Parse an input from the command line as a Ghost Admin API key
fn parse_key(arg: &str) -> Result<String> {
    // Use the key provided on the command-line
    if !arg.is_empty() {
        return validate_key(arg);
    }

    // If not, check if it's provided as an environment variable
    if let Ok(env_key) = std::env::var("STENCILA_GHOST_KEY") {
        return validate_key(&env_key);
    }

    // Lastly, check the keyring.
    secrets::get(SECRET_NAME)
}

fn parse_host(arg: &str) -> Result<Host> {
    // Question mark converts between error types
    Ok(Host::parse(arg)?)
}
