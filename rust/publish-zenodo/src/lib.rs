// TODO: use the [Research Organization Registry (ROR)](https://ror.org/) database to gather affiliations for authors


use std::{path::PathBuf, str::FromStr, sync::{Arc, Mutex}};

use codec::{schema::{Primitive, PropertyValue, PropertyValueOrString}, Codec};
use cli_utils::{parse_host, ToStdout};
use cli_utils::parse_host;
use cli_utils::{parse_host, ToStdout};
use common::{
    clap::{self, Parser}, eyre::{bail, OptionExt, Result}, reqwest::Client, serde, serde_json::{json, Value}, tempfile, tokio, tracing,
};

use document::{schema, CommandWait, Document, EncodeOptions, Format, schema::Node};
use codec_swb::SwbCodec;

mod metadata_extraction;

fn parse_date(input: &str) -> Result<schema::Date> {
    Ok(schema::Date::from_str(input)?)
}


/// Items within Zenodo's controlled vocabulary of accepted types of publication
#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[serde(crate = "common::serde", rename_all = "lowercase")]
enum PublicationType {
    AnnotationCollection,
    Book,
    Section,
    ConferencePaper,
    DataManagementPlan,
    Article,
    Patent,
    #[default]
    Preprint,
    Deliverable,
    Milestone,
    Proposal,
    Report,
    SoftwareDocumentation,
    TaxonomicTreatment,
    TechnicalNote,
    Thesis,
    WorkingPaper,
    Other,
}

/// Publish to Zenodo
/// 
/// Metadata for the deposition is extracted from the metadata within the
/// document.
/// 
/// Once uploaded, you will prompted to review the deposition and publish from
/// Zenodo's web interface. If you wish to skip the review process and publish
/// immediately, then use the `--force` flag.
/// 
/// By default, Zenodo's testing server is used. This means that DOIs created
/// will not resolve. To upload to Zenodo's public-facing production server, use
/// the `--zenodo` flag. To upload to a specific server instance, use
/// `--server`.
/// 
/// Unless told otherwise, your deposit will be given the publication type "preprint".
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to location of what to publish
    #[arg(default_value = ".")]
    #[arg(display_order(0))]
    path: PathBuf,

    /// Zenodo authentication token
    /// 
    /// To create one, log into Zenodo, visit your account's page, then click
    /// "Applications", followed by "+ New Token" within the "Personal access
    /// tokens" section. Give the token a name and enable the "deposit:actions" the scope.
    /// 
    /// Enable the "deposit:write" scope to enable the `--force` flag.
    #[arg(long, env = "STENCILA_ZENODO_TOKEN")]
    #[arg(help_heading("Zenodo Settings"))]
    #[arg(display_order(1))]
    token: Option<String>,

    // Server selection options

    /// Publish to Zenodo's testing server
    #[arg(group = "zenodo_server")]
    #[arg(long, default_value_t = true)]
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    sandbox: bool,

    /// Publish to Zenodo's public-facing server
    /// 
    /// [default]
    #[arg(group = "zenodo_server")]
    #[arg(long)]
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    zenodo: bool,

    /// Publish to a specific Zenodo server
    #[arg(group = "zenodo_server")]
    #[arg(long, value_parser = parse_host)]
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    server: Option<url::Host>,

    // Resource type options

    /// Upload document as a "lesson"
    #[arg(group = "resource_type")]
    #[arg(long)]
    #[arg(conflicts_with_all = ["publication", "publication_type"])]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    lesson: bool,

    /// Upload document as a "publication"
    /// 
    /// [default]
    #[arg(group = "resource_type")]
    #[arg(long, default_value_t = true)]
    #[arg(default_value_if("lesson", common::clap::builder::ArgPredicate::IsPresent, "false"))]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    publication: bool,

    /// Reserve a DOI for the deposition
    #[arg(long)]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    reserve_doi: bool,

    /// Publication date
    /// 
    /// When omitted, Zenodo will use today's date.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(value_parser = parse_date)]
    publication_date: Option<schema::Date>,

    /// Title to use for the deposit
    /// 
    /// Required when the information is not available within the document.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    title: Option<String>,

    /// Description to use within the deposition
    /// 
    /// Required when the information is not available within the document.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    description: Option<String>,

    ///  One of the publication types from Zenodo's controlled vocabulary.
    /// 
    /// [default: preprint]
    #[arg(long)]
    #[arg(required_if_eq("publication", "true"))]
    #[arg(default_value_if("publication", "true", "preprint"))]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    publication_type: Option<PublicationType>,

    /// Force publish the deposition immediately
    /// 
    /// Requires that access token provided by the `--token` option has the "deposit:write" scope.
    #[arg(long)]
    force: bool,

    /// Dry run mode - no actual upload
    #[arg(long)]
    dry_run: bool,
}

/// Create an article's filename from article title by:
/// - Converting to lowercase
/// - Replacing non-alphanumeric chars with hyphens
/// - Collapsing multiple hyphens
/// - Trimming hyphens from start/end
/// - Defaulting to "untitled"
fn make_filename(title: &Option<Vec<schema::Inline>>) -> Result<String> {
    let title_text = title
        .as_ref()
        .map(|t| codec_text::to_text(t))
        .unwrap_or_else(|| "untitled".to_string());

    let mut result = String::new();
    let mut last_was_hyphen = true; // To prevent starting with hyphen
    
    for c in title_text.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            result.push(c);
            last_was_hyphen = false;
        } else if !last_was_hyphen {
            result.push('-');
            last_was_hyphen = true;
        }
    }

    // Trim trailing hyphen if exists
    if result.ends_with('-') {
        result.pop();
    }

    // Return "untitled" if no valid chars were found
    if result.is_empty() {
        Ok("untitled".to_string())
    } else {
        Ok(result)
    }
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Check preconditions first
        
        // Validate API token is available 
        let token = self.token
            .as_ref()
            // .or_else(|| secrets::env_or_get("ZENODO_TOKEN").ok()) // TODO: use OS keyring via Stencila's secrets module
            .ok_or_eyre("Zenodo API token not provided and not set as a secret")?;

        // Validate input path
        if !self.path.exists() {
            bail!("Path does not exist: {}", self.path.display())
        }

        if !self.path.is_file() {
            bail!("Only publishing files is currently supported")
        }

        // Open and compile document
        let doc = Document::open(&self.path).await?;
        doc.compile(CommandWait::Yes).await?;

        let doi = doc
            .inspect(|root| {
                let Node::Article(article) = root else {
                    return None;
                };

                let Some(ids) = &article.options.identifiers else {
                    return None;
                };

                for id in ids {
                    if let Some(doi) = metadata_extraction::extract_doi(id) {
                        return Some(doi.to_string());
                    }
                }

                None
            })
            .await;

        // Get the root node and ensure it's an Article
        let root = doc.root().await;
        let Node::Article(ref article) = &root else {
            tracing::error!("Internal error: Document root is not an Article");
            bail!("Document root must be an Article");
        };

         let doc_updated = Arc::new(Mutex::new(false));
         let doc_updated_ = Arc::clone(&doc_updated);
         
         doc.mutate(move |root| {
             let Node::Article(article) = root else { return };

             let Ok(mut doc_updated) = doc_updated_.lock() else {
                tracing::error!("internal error: failed to acquire lock");
                return;
            };
            
            if let Some(desc) = &self.description {
                article.description = Some(desc.clone());
                *doc_updated = true;
            }

            if *doc_updated {
                let today = common::chrono::Utc::now().naive_utc().to_string();
                let date_modified = schema::Date::new(today);
                article.date_modified = Some(date_modified);
            }
        })
        .await;

        match doc_updated.lock() {
            Ok(update) if *update == true => doc.save(CommandWait::Yes).await?,
            Ok(_) => (),
            Err(_) => tracing::error!("internal error: failed to acquire lock. Proceeding without modifying document."),    
        };

        // Determine server URL
        let server_url = if let Some(server) = self.server {
            server
        } else if self.zenodo {
            url::Host::parse("zenodo.org")?
        } else {
            url::Host::parse("sandbox.zenodo.org")?
        };

        let metadata_from_doc = doc.inspect(|root| {
            let Node::Article(article) = root else { return None };

            let title = article.title.as_ref().map(|t| codec_text::to_text(t)).or_else(|| self.title.clone()).unwrap_or_else(|| { "Untitled".to_string() });
            let description = article.description.as_ref().map(|t| codec_text::to_text(t));
            let mut creators = Vec::new();

            if let Some(authors) = &article.authors {
    
                for author in authors {
                    match author {
                        schema::Author::Person(person) => {
                            let mut affiliation = None;
                            
                            // Zenodo expects ORCIDs to be numbers and hyphens only, e.g,
                            // 0000-0000-0000-0000, although it the last digit can be a trailing
                            // X to indicate a checksum
                            let mut orcid = None;

                            let name = metadata_extraction::extract_name(&person);
    
                            // FIXME: for some reason, I can't find the ORCIDs or affiliations
    
                            // find orcid in list of identifiers
                            person.options.identifiers.as_ref().map(|ids| {
                                for id in ids {
                                    tracing::debug!(property_value = ?id, "extracting orcid from id");
                                    if orcid.is_none() {
                                        orcid = metadata_extraction::extract_orcid(&id);
                                    }
                                }
                            });
    
                            if let Some(mut affs) = metadata_extraction::extract_affiliations(&person) {
                                affiliation = affs.next();
    
                                if affiliation.is_some() && affs.next().is_some() {
                                    let name_part = name.as_ref().map(|name| { format!("({name}) ") }).unwrap_or_default();
                                    let org_part = affiliation.as_ref().map(|org| { format!("({org})") }).unwrap_or_default();
    
                                    tracing::warn!("The author {name_part}has multiple affiliations. Only the first {org_part}can be added programmatically by Stencila. Please edit the record within Zenodo before publication to make corrections.");
                                    break;
                                }
                            }

                            let creator = json!({
                                "name": name,
                                "affiliation": affiliation,
                                "orcid": orcid,
                            });
                            creators.push(creator);       
                        },
                        _ => (),
                    }
                }
            }

            Some((title, description, creators))
        }).await;

        let mut deposit = json!({ "metadata": json!({})});

        if let Some((title, description, creators)) = metadata_from_doc {
            deposit["metadata"]["title"] = json!(title);
            deposit["metadata"]["description"] = json!(description);
            deposit["metadata"]["creators"] = json!(creators);
        }
        if let Some(title_from_args) = self.title {
            deposit["metadata"]["title"] = json!(title_from_args);
        }

        if self.lesson {
            deposit["metadata"]["upload_type"] = json!("lesson");
        } else {
            deposit["metadata"]["upload_type"] = json!("publication");
        };

        if self.publication {
            deposit["metadata"]["publication_type"] = json!(self.publication_type.unwrap_or_default());
        }

        match (doi, self.reserve_doi) {
            (Some(doi), true) => tracing::warn!("Document already has a DOI ({doi}). The --reserve-doi flag will be ignored."),
            (Some(doi), false) => deposit["metadata"]["doi"] = json!(doi),
            (None, true) => deposit["metadata"]["prereserve_doi"] = json!(true),
            (None, false) => (),
        }

        tracing::debug!(metadata = ?deposit, "Deposit metadata provided to Zenodo");

        // Create HTTP client
        let client = Client::new();

        // Create deposition
        let deposition_response = client
            .post(format!("https://{server_url}/api/deposit/depositions"))
            .query(&[("access_token", &token)])
            .json(&deposit)
            .send()
            .await?;

        tracing::info!(response = ?deposition_response, "Deposit creation response");

        // permissions error
        if deposition_response.status().as_u16() == 403 {
            let mut msg = String::from("hint: Check that the access token is correct and has the necessary scope");

            if self.force {
                msg.push_str("s (`deposit:actions` and `deposit:write`) enabled." );
            } else {
                msg.push_str(" (`deposit:actions`) enabled." );
            }

            tracing::info!(msg);
        }

        if !&deposition_response.status().is_success() {
            let data: Value = deposition_response.json().await?;

            if let Some(Value::Array(errors))  = data.get("errors") {
                for error in errors {
                    if let (Some(Value::String(field)), Some(Value::Array(messages))) = (error.get("field"), error.get("messages")) {
                        if field == "metadata.description" {
                            for message in messages {
                                message.as_str().map(|msg| {
                                    if msg == "Field may not be null." {
                                        tracing::info!("hint: Provide a description with the --description flag.");
                                    }
                                });
                            }
                        };
                    };
                }
            }

            bail!("Failed to create deposition: {:?}", data.as_str());
        }

        let deposition: Value = deposition_response.json().await?;
        tracing::debug!(deposition = ?deposition, "Response from creating deposit");
        let deposition_id = deposition["id"]
            .as_u64()
            .ok_or_eyre("No deposition ID in response in the response from Zenodo")?;

        let reserved_doi = deposition["metadata"]["prereserve_doi"]["doi"].as_str();
        let deposition_url = deposition["links"]["self"].as_str().ok_or_eyre("No deposition URL provided in the response from Zenodo")?;

        doc.mutate(move |root| {
            let Node::Article(article) = root else { return };

            let zenodo_property = PropertyValueOrString::String(deposition_url.to_string());
            let mut zenodo_ids = vec![zenodo_property];

            if let Some(doi) = reserved_doi {
                let property = PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some("https://registry.identifiers.org/registry/doi".into()),
                    value: Primitive::String(doi.into()),
                    ..Default::default()
                });

                zenodo_ids.push(property);
            }
        
            match article.options.identifiers.as_mut() {
                Some(ids) => ids.extend(zenodo_ids.into_iter()),
                None => article.options.identifiers = Some(zenodo_ids),
            };

            let now = common::chrono::Utc::now().naive_utc().to_string();
            let date_modified = schema::Date::new(now);
            article.date_modified = Some(date_modified);
        })
        .await;

        doc.save(CommandWait::Yes).await?;

        // Create a temporary directory and file for the SWB
        let temp_dir = tempfile::tempdir()?;
        let filename = make_filename(&article.title)?;
        let swb_path = temp_dir.path().join(format!("{}.swb", filename));

        // Create the SWB bundle
        SwbCodec::default()
            .to_path(
                &root,
                &swb_path, 
                Some(EncodeOptions {
                    format: Some(Format::Swb),
                    standalone: Some(true),
                    ..Default::default()
                }),
            )
            .await?;

        if self.dry_run {
            tracing::info!("Dry run complete - bundle created at {}", swb_path.display());
            return Ok(());
        }

        // Get bucket URL for file upload
        let bucket_url = deposition["links"]["bucket"]
            .as_str()
            .ok_or_eyre("No bucket URL in response")?;

        // Upload the SWB file using same filename
        let file_name = format!("{}.swb", filename);
        let url: String = format!("{}/{}", bucket_url, file_name);
        tracing::info!(url = url, file = file_name, "Uploading deposit");
        let upload_response = client
            .put(&format!("{}/{}", bucket_url, file_name))
            .query(&[("access_token", &token)])
            .body(tokio::fs::read(&swb_path).await?)
            .send()
            .await?;

        if !upload_response.status().is_success() {
            tracing::error!(response = ?upload_response, file = ?file_name, "Failed to upload");
            bail!("Failed to upload file: {}", upload_response.text().await?);
        }

        let deposition_url = deposition["links"]["html"].as_str().ok_or_eyre("No deposit URL provided by Zenodo.")?;
        
        if self.force {
            // Publish the deposition
            let publish_response = client
                .post(format!(
                    "{}/api/deposit/depositions/{}/actions/publish",
                    server_url, deposition_id
                ))
                .query(&[("access_token", &token)])
                .send()
                .await?;

            if !publish_response.status().is_success() {
                bail!("Failed to publish deposition: {}", publish_response.text().await?);
            }

            tracing::info!("Created published deposition at {deposition_url}.");
        } else {

            let mut msg = format!("Successfully published document as a draft deposit to {deposition_url}");
            if let Some(doi) = reserved_doi {
                msg.push_str(&format!(" with the doi:{doi} pre-reserved"));
            }
            msg.push('.');
            cli_utils::message!("{}", msg).to_stdout();
            cli_utils::message!("Review and publish manually, or use --force to publish directly");
        }

        Ok(())
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn defaults_to_sandbox() {
        let args = Cli::parse_from(&["publish-zenodo"]);
        assert!(args.sandbox);
    }

    /// Tests that the publication type defaults to pre-print when --publication is set
    #[test]
    fn publication_type_defaults_to_preprint() {
        let args = Cli::parse_from(&["publish-zenodo", "--publication", "some.smd"]);
        println!("{args:#?}");
        assert!(matches!(args.publication_type, Some(PublicationType::Preprint)));
    }

    /// Tests that the publication type defaults to pre-print when --publication is set
    #[test]
    fn upload_type_lesson_does_not_require_publication_type() {
        let args = Cli::parse_from(&["publish-zenodo", "--lesson", "some.smd"]);
        println!("{args:#?}");
        assert!(args.sandbox);
        assert!(matches!(args.publication_type, None));
        assert!(!args.publication);
    }
}