use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken as jwt;
use url::Host;

use common::{
    clap::{self, Parser, ValueEnum},
    eyre::{bail, eyre, Context, OptionExt, Result},
    reqwest::{Client, Response, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    tracing,
};
use document::{
    schema::{Node, PropertyValueOrString},
    CommandWait, DecodeOptions, Document, EncodeOptions, Format,
};

const SECRET_NAME: &str = "GHOST_ADMIN_API_KEY";

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
enum PublishAction {
    #[default]
    Push,
    Pull,
}

/// Publish to Ghost
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file or directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// The action to perform with the file
    ///
    /// Defaults to push (i.e. create if not yet published, update if does).
    /// Use `pull` to update a file that has already been published.
    action: Option<PublishAction>,

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
    #[arg(long, conflicts_with = "post", default_value_t = true)]
    page: bool,

    /// Create a post
    #[arg(long, conflicts_with = "page")]
    post: bool,

    /// Dry run test
    ///
    /// When set, stencila will perform the document conversion but skip the publication to Ghost.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

impl Cli {
    /// Run the CLI command
    pub async fn run(self) -> Result<()> {
        if !self.path.exists() {
            bail!("Path does not exist: {}", self.path.display())
        }

        if !self.path.is_file() {
            bail!("Only publishing files is currently supported")
        }

        // Open and compile document
        let doc = Document::open(&self.path).await?;
        doc.compile(CommandWait::Yes).await?;

        let base_url = format!("https://{}/ghost/api/admin/", self.ghost);

        // Get Ghost URL of the document, if any.
        // Must be a URL on the current Ghost host
        let doc_url = doc
            .inspect(|root| {
                let mut doc_url = None;
                if let Node::Article(article) = root {
                    if let Some(ids) = &article.options.identifiers {
                        for id in ids {
                            if let PropertyValueOrString::String(id) = id {
                                if id.starts_with(&base_url) {
                                    doc_url = Some(id.clone())
                                }
                            }
                        }
                    }
                }
                doc_url
            })
            .await;

        // Dispatch to method based on action and presence of existing doc URL
        let action = self.action.unwrap_or_default();
        match (action, doc_url) {
            (PublishAction::Push, None) => self.post(doc, base_url).await,
            (PublishAction::Push, Some(doc_url)) => self.put(doc, doc_url).await,
            (PublishAction::Pull, Some(doc_url)) => self.get(doc, doc_url).await,
            (PublishAction::Pull, None) => bail!(
                "Unable to pull document, does not have document on Ghost host `{}`",
                self.ghost
            ),
        }
    }

    /// POST a document (ie publish post or page for first time)
    #[tracing::instrument(skip(doc))]
    async fn post(&self, doc: Document, base_url: String) -> Result<()> {
        tracing::trace!("Publishing document to {base_url}");

        // Get document title and other metadata
        let (title,) = doc_metadata(&doc).await;

        // Dump document to a Lexical (Ghost's Dialect) string
        let lexical = doc
            .dump(
                Format::Koenig,
                Some(EncodeOptions {
                    // TODO: The option for "just one big HTML card" so go here
                    standalone: Some(false),
                    ..Default::default()
                }),
            )
            .await?;

        // Generate JWT for request
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Construct JSON payload
        // See https://ghost.org/docs/admin-api/#creating-a-post
        // and https://github.com/stencila/stencila/issues/2481 for how they map to CLI args
        let root_key = self.root_key()?;
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
            .post(format!("{}{}/", base_url, root_key))
            .header("Authorization", format!("Ghost {token}"))
            .json(&payload)
            .send()
            .await?;

        // Handle the response...
        if let StatusCode::CREATED = response.status() {
            // Get the URL of the newly created Ghost page/post
            let Some(location) = response.headers().get("location") else {
                tracing::error!(resp = ?response, "POST succeeded, but Location header unavailable");
                bail!("Uploading the document to Ghost appears to have succeeded, but Ghost did not provide the new URL. Check Ghost Admin for the new draft.");
            };
            let url = location
                .to_str()
                .context("interpreting Location HTTP header")?
                .to_string();

            // Add the URL to the article's identifiers
            doc.mutate(move |root| {
                if let Node::Article(article) = root {
                    let identifier = PropertyValueOrString::String(url.clone());
                    match article.options.identifiers.as_mut() {
                        Some(ids) => ids.push(identifier),
                        None => article.options.identifiers = Some(vec![identifier]),
                    }
                }
            })
            .await;

            // Save the document to disk
            doc.save(CommandWait::Yes).await
        } else {
            error_for_response(response).await
        }
    }

    /// PUT a document (ie update post or page from local file)
    #[tracing::instrument(skip(doc))]
    async fn put(&self, doc: Document, doc_url: String) -> Result<()> {
        tracing::trace!("Updating document {doc_url}");

        // Get document title and other metadata
        let (title,) = doc_metadata(&doc).await;

        // Dump document to a Lexical (Ghost's Dialect) string
        let lexical = doc
            .dump(
                Format::Koenig,
                Some(EncodeOptions {
                    // TODO: The option for "just one big HTML card" so go here
                    standalone: Some(false),
                    ..Default::default()
                }),
            )
            .await?;

        // Generate JWT for request
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Get the most recent updated_at to avoid a "Saving failed! Someone else is editing this post." error
        let response = Client::new()
            .get(&doc_url)
            .header("Authorization", format!("Ghost {token}"))
            .send()
            .await?;

        let json: serde_json::Value = if let StatusCode::OK = response.status() {
            response.json().await?
        } else {
            return error_for_response(response).await;
        };

        let root_key = self.root_key()?;
        let updated_at = json
            .get(root_key)
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("updated_at"))
            .and_then(|value| value.as_str())
            .ok_or_eyre("Response has unexpected structure")?;

        // Construct JSON payload
        let root_key = self.root_key()?;
        let payload = serde_json::json!({
            root_key : [
                json!({
                    "title": title,
                    "lexical": lexical,
                    "updated_at": updated_at,
                })
            ]
        });

        // Return early if this is just a dry run
        if self.dry_run {
            return Ok(());
        }

        // Send the request
        let response = Client::new()
            .put(&doc_url)
            .header("Authorization", format!("Ghost {token}"))
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::debug!("Update succeeded");
            Ok(())
        } else {
            error_for_response(response).await
        }
    }

    /// GET a document (ie update a local file from remote)
    #[tracing::instrument(skip(doc))]
    async fn get(&self, doc: Document, doc_url: String) -> Result<()> {
        tracing::trace!("Getting document {doc_url}");

        // Generate JWT for request
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Return early if this is just a dry run
        if self.dry_run {
            return Ok(());
        }

        // Send the request
        let response = Client::new()
            .get(&doc_url)
            .header("Authorization", format!("Ghost {token}"))
            .send()
            .await?;

        let json: serde_json::Value = if let StatusCode::OK = response.status() {
            response.json().await?
        } else {
            return error_for_response(response).await;
        };

        let root_key = self.root_key()?;
        let lexical = json
            .get(root_key)
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("lexical"))
            .and_then(|value| value.as_str())
            .ok_or_eyre("Response has unexpected structure")?;

        // Merge the Lexical into the document
        doc.load(
            lexical,
            Some(DecodeOptions {
                format: Some(Format::Koenig),
                ..Default::default()
            }),
            // TODO: consider constructing a Vec<AuthorRole> here so
            // that authorship can be assigned for the merge
            None,
        )
        .await?;

        // Save the document to disk
        doc.save(CommandWait::Yes).await
    }

    /// Get the "root_key" for the page or post
    fn root_key(&self) -> Result<&str> {
        if self.post {
            Ok("posts")
        } else if self.page {
            Ok("pages")
        } else {
            bail!("Please use --post or --page flag");
        }
    }
}

/// Parse an input from the command line as a Ghost host
fn parse_host(arg: &str) -> Result<Host> {
    // Question mark converts between error types
    Ok(Host::parse(arg)?)
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

// Validate that a key looks like a Ghost Admin API key
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

    fn only_hex(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_ascii_lowercase() && c.is_ascii_hexdigit())
    }
    if !only_hex(id) || !only_hex(secret) {
        tracing::warn!("Ghost Admin API key may be invalid; should only contain lowercase hexadecimal characters");
    }

    Ok(key.to_string())
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct Claims {
    // "Audience", e.g. a URL in the Ghost instance
    aud: String,
    iat: u64,
    exp: u64,
}

/// Generate a Ghost JWT
fn generate_jwt(key: &str) -> Result<String> {
    let Some((id, secret)) = key.split_once(':') else {
        return Err(eyre!("invalid Ghost Admin API key")); // should never happen because validated on entry
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

/// Generate an error for an unsuccessful response
///
/// Attempts to extract error message from JSON response, and if
/// that fails, displays the body text.
async fn error_for_response(response: Response) -> Result<()> {
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
        bail!("HTTP {code}")
    }
}

/// Get document metadata
///
/// TODO: other metadata such as authors, excerpt (from abstract?)
/// could be obtained in a similar way and returned from this function
async fn doc_metadata(doc: &Document) -> (String,) {
    doc.inspect(|root: &Node| {
        let mut title = String::from("Untitled");

        if let Node::Article(article) = root {
            if let Some(inlines) = &article.title {
                title = codec_text::to_text(inlines);
            }
        }

        (title,)
    })
    .await
}
