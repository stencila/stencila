use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken as jwt;
use url::Host;

use common::{
    chrono::{DateTime, Utc},
    clap::{self, Parser},
    eyre::{bail, eyre, Context, OptionExt, Result},
    reqwest::{Client, Response, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    strum::Display,
    tracing,
};
use document::{
    schema::{shortcuts::t, Node, PropertyValueOrString},
    CommandWait, DecodeOptions, Document, EncodeOptions, Format, LossesResponse,
};

const KEY_ENV_VAR: &str = "STENCILA_GHOST_KEY";
const SECRET_NAME: &str = "GHOST_ADMIN_API_KEY";

/// Publish to Ghost
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file or directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// The Ghost domain
    ///
    /// This is the domain name of your Ghost instance, with an optional port.
    ///
    /// Not required when pushing or pulling an existing post or page from
    /// Ghost but if supplied only document `identifiers` with this host
    /// will be used.
    #[arg(long, env = "STENCILA_GHOST_DOMAIN", value_parser = parse_host)]
    ghost: Option<Host>,

    /// The Ghost Admin API key
    ///
    /// To create one, create a new Custom Integration under
    /// the Integrations screen in Ghost Admin. Use the Admin API Key,
    /// rather than the Content API Key.
    ///
    /// You can also set the key as a secret so that it does not need to
    /// be entered here each time: `stencila secrets set GHOST_ADMIN_API_KEY`.
    #[arg(long, env = KEY_ENV_VAR, value_parser = parse_key)]
    key: Option<String>,

    /// Create a page
    ///
    /// Does not apply when pushing to, or pulling from, and existing
    /// Ghost resource.
    #[arg(long, conflicts_with = "post", default_value_t = true)]
    page: bool,

    /// Create a post
    ///
    /// Does not apply when pushing to, or pulling from, and existing
    /// Ghost resource.
    #[arg(long, conflicts_with = "page")]
    post: bool,

    /// Create or update Ghost post or page from a file
    #[arg(long, conflicts_with = "pull", default_value_t = true)]
    push: bool,

    /// Update file from an existing Ghost post or page
    #[arg(long, conflicts_with = "push")]
    pull: bool,

    /// Push as draft
    #[arg(
        long,
        group = "publish_type",
        requires = "push",
        default_value_t = true
    )]
    draft: bool,

    /// Publish pushed, page or post
    #[arg(long, group = "publish_type", requires = "push")]
    publish: bool,

    /// schedule pushed, page or post
    #[arg(long, group = "publish_type", requires = "push")]
    schedule: Option<DateTime<Utc>>,

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

        // Get Ghost URL of the document, if any.
        let doc_url = doc
            .inspect(|root| {
                let Node::Article(article) = root else {
                    return None;
                };

                let Some(ids) = &article.options.identifiers else {
                    return None;
                };

                for id in ids {
                    if let PropertyValueOrString::String(id) = id {
                        if let Some(host) = &self.ghost {
                            // If a host is provided then return the first URL on that host
                            if id.starts_with(&format!("https://{host}/ghost/api/admin/")) {
                                return Some(id.clone());
                            }
                        } else if id.starts_with("https://") && id.contains("/ghost/api/admin/") {
                            // Otherwise, return the first URL on any Ghost host
                            return Some(id.clone());
                        }
                    }
                }

                None
            })
            .await;

        // Dispatch to method based on action and presence of existing doc URL
        match (self.push, self.pull, doc_url) {
            // Pull (first, because push defaults to true)
            (_, true, Some(doc_url)) => self.get(doc, doc_url).await,
            (_, true, None) => {
                bail!("Unable to pull document, does not have corresponding Ghost identifier")
            }

            // Push
            (true, _, None) => self.create(doc).await,
            (true, _, Some(doc_url)) => self.put(doc, doc_url).await,

            _ => bail!("UNexpected combination of --pull and --push"),
        }
    }

    /// Create a post or page by POSTing it to the Ghost API
    #[tracing::instrument(skip(doc))]
    async fn create(&self, doc: Document) -> Result<()> {
        // Require a host
        let Some(host) = &self.ghost else {
            bail!("File does not have an identifier for Ghost so the ---ghost option must be provided");
        };
        let base_url = format!("https://{host}/ghost/api/admin/");

        tracing::trace!("Publishing document to {base_url}");

        // Determine the type of resource
        let resource_type = if self.post {
            ResourceType::Post
        } else if self.page {
            ResourceType::Page
        } else {
            // Default is `--post = true` so this should not occur
            bail!("Please use either the --post or --page flag");
        };

        // Generate JWT for request
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Status of page or post
        let status = if self.publish {
            Some(Status::Published)
        } else if self.schedule.is_some() {
            if self.schedule <= Some(Utc::now()) {
                bail!(
                    "Scheduled time must be in the future, current time:{:?} , scheduled time:{:?}",
                    self.schedule,
                    Utc::now()
                );
            }
            Some(Status::Scheduled)
        } else if self.draft {
            Some(Status::Draft)
        } else {
            None
        };

        // Construct the POST payload
        let payload =
            Payload::from_doc(resource_type, &doc, None, status, self.schedule).await?;

        // Return early if this is just a dry run
        if self.dry_run {
            return Ok(());
        }

        // Send the request
        let response = Client::new()
            .post(format!("{}{}s/", base_url, resource_type))
            .header("Authorization", format!("Ghost {token}"))
            .json(&payload)
            .send()
            .await?;

        // Error early if not created
        if response.status() != StatusCode::CREATED {
            return error_for_response(response).await;
        }

        // Get the URL of the newly created Ghost page/post
        let Some(location) = response.headers().get("location") else {
            tracing::error!(resp = ?response, "POST succeeded, but Location header unavailable");
            bail!("Uploading the document to Ghost appears to have succeeded, but Ghost did not provide the new URL. Check Ghost Admin for the new draft.");
        };
        let doc_url = location
            .to_str()
            .context("interpreting Location HTTP header")?
            .to_string();

        // Add the URL to the article's identifiers
        let url = doc_url.clone();
        doc.mutate(move |root| {
            let Node::Article(article) = root else { return };

            let identifier = PropertyValueOrString::String(url.clone());
            match article.options.identifiers.as_mut() {
                Some(ids) => ids.push(identifier),
                None => article.options.identifiers = Some(vec![identifier]),
            }
        })
        .await;

        // Save the document to disk
        doc.save(CommandWait::Yes).await?;

        tracing::info!(
            "Successfully created {doc_url} from {}",
            doc.file_name().unwrap_or("document")
        );

        Ok(())
    }

    /// Update a Ghost post or page from local file
    #[tracing::instrument(skip(doc))]
    async fn put(&self, doc: Document, doc_url: String) -> Result<()> {
        tracing::trace!("Updating document {doc_url}");

        // Determine the type of payload from the document URL
        let resource_type = if doc_url.contains("/posts/") {
            ResourceType::Post
        } else if doc_url.contains("/pages/") {
            ResourceType::Page
        } else {
            bail!("Unable to determine whether to update post or page from URL: {doc_url}");
        };

        // Generate JWT for requests
        let token = generate_jwt(&self.key).context("generating JWT")?;

        // Return early if this is just a dry run
        if self.dry_run {
            return Ok(());
        }

        // Get the most recent `updated_at`` to avoid the error
        // "Saving failed! Someone else is editing this post."
        let response = Client::new()
            .get(&doc_url)
            .header("Authorization", format!("Ghost {token}"))
            .send()
            .await?;

        // Parse response to get `updated_at`
        let mut payload: Payload = if let StatusCode::OK = response.status() {
            response.json().await?
        } else {
            return error_for_response(response).await;
        };
        let Resource { updated_at, .. } = payload.resource()?;

        // Status of page or post
        let status = if self.publish {
            Some(Status::Published)
        } else if self.schedule.is_some() {
            if self.schedule <= Some(Utc::now()) {
                bail!(
                    "Scheduled time must be in the future, current time:{:?} , scheduled time:{:?}",
                    self.schedule,
                    Utc::now()
                );
            }
            Some(Status::Scheduled)
        } else if self.draft {
            Some(Status::Draft)
        } else {
            None
        };

        // Construct the PUT payload with the latest `updated_at`
        let payload =
            Payload::from_doc(resource_type, &doc, updated_at, status, self.schedule).await?;

        // Send the request
        let response = Client::new()
            .put(&doc_url)
            .header("Authorization", format!("Ghost {token}"))
            .json(&payload)
            .send()
            .await?;

        // Handle response
        if response.status().is_success() {
            tracing::info!(
                "Successfully updated {doc_url} from {}",
                doc.file_name().unwrap_or("document")
            );
            Ok(())
        } else {
            error_for_response(response).await
        }
    }

    /// Update a local file from a Ghost post or page
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

        // Parse the response into a resource
        let mut payload: Payload = if let StatusCode::OK = response.status() {
            response.json().await?
        } else {
            return error_for_response(response).await;
        };
        let Resource { title, lexical, .. } = payload.resource()?;

        // Update title etc
        // TODO: consider other properties that might be appropriate to update from Ghost
        doc.mutate(|root| {
            let Node::Article(article) = root else { return };

            article.title = title.as_ref().map(|title| vec![t(title)]);
        })
        .await;

        // Merge the Lexical into the document
        if let Some(lexical) = &lexical {
            doc.load(
                lexical,
                Some(DecodeOptions {
                    format: Some(Format::Koenig),
                    losses: LossesResponse::Debug,
                    ..Default::default()
                }),
                // TODO: consider constructing a Vec<AuthorRole> here so
                // that authorship can be assigned for the merge
                None,
            )
            .await?;
        }

        // Save the document to disk
        doc.save(CommandWait::Yes).await?;

        tracing::info!(
            "Successfully updated {} from {doc_url}",
            doc.file_name().unwrap_or("document")
        );

        Ok(())
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
    if let Ok(env_key) = std::env::var(KEY_ENV_VAR) {
        return validate_key(&env_key);
    }

    // Should not happen because this function is only called if
    // an argument is provided
    bail!("No key provided")
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
fn generate_jwt(key: &Option<String>) -> Result<String> {
    // Use the key provided on CLI or in env, otherwise try to get secret from env or store
    let key = key
        .clone()
        .or_else(|| secrets::env_or_get(SECRET_NAME).ok())
        .ok_or_eyre("Ghost Admin API key not provided and not set as a secret")?;

    let Some((id, secret)) = key.split_once(':') else {
        bail!("Invalid Ghost Admin API key"); // should never happen because validated on entry
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

/// A Ghost page or post
///
/// This schema for this is available at:
///
/// https://github.com/TryGhost/SDK/blob/main/packages/admin-api-schema/lib/schemas/posts.json
/// https://github.com/TryGhost/SDK/blob/main/packages/admin-api-schema/lib/schemas/pages.json
///
/// Note also that there are `*.-add.json` (for creating) and `*-edit.json` (for updating).
/// At present we are keeping things simple and using one `struct` for all these use
/// cases but some specialization may be required in the future.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct Resource {
    title: Option<String>, // Required for creating
    lexical: Option<String>,
    status: Option<Status>,
    updated_at: Option<String>,          // Required for updating
    published_at: Option<DateTime<Utc>>, // Required for scheduling
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
enum Status {
    Draft,
    Published,
    Scheduled,
}

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "lowercase")]
enum ResourceType {
    Post,
    Page,
}

/// A payload from the Ghost Admin API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
enum Payload {
    Posts(Vec<Resource>),
    Pages(Vec<Resource>),
}

impl Payload {
    /// Create a payload from a document
    async fn from_doc(
        resource_type: ResourceType,
        doc: &Document,
        updated_at: Option<String>,
        status: Option<Status>,
        published_at: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        // Get document title and other metadata
        // TODO: other metadata such as authors, excerpt (from abstract?)
        // could be obtained in a similar way and returned from this function
        let (title,) = doc
            .inspect(|root: &Node| {
                let mut title = None;

                if let Node::Article(article) = root {
                    if let Some(inlines) = &article.title {
                        title = Some(codec_text::to_text(inlines));
                    }
                }

                (title,)
            })
            .await;

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

        let resource = Resource {
            title: title.or_else(|| Some("Untitled".into())),
            lexical: Some(lexical),
            updated_at,
            status,
            published_at,
            ..Default::default()
        };

        Ok(match resource_type {
            ResourceType::Post => Payload::Posts(vec![resource]),
            ResourceType::Page => Payload::Pages(vec![resource]),
        })
    }

    /// Get the first resource from a payload
    ///
    /// Used when GETing from the API to extract the page or post.
    fn resource(&mut self) -> Result<Resource> {
        match self {
            Payload::Posts(posts) => posts.pop(),
            Payload::Pages(pages) => pages.pop(),
        }
        .ok_or_eyre("Payload does not have any page or post")
    }
}
