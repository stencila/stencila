// TODO: use the [Research Organization Registry (ROR)](https://ror.org/) database to gather affiliations for authors

use std::{
    io::IsTerminal,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use cli_utils::{parse_host, ToStdout};
use codec::{
    schema::{Primitive, PropertyValue, PropertyValueOrString},
    Codec,
};
use common::{
    clap::{self, builder::ArgPredicate, Parser},
    eyre::{bail, OptionExt, Result},
    reqwest::Client,
    serde,
    serde_json::{json, Value},
    tempfile, tokio, tracing,
};

use codec_swb::SwbCodec;
use color_print::cstr;
use document::{schema, schema::Node, CommandWait, Document, EncodeOptions, Format};

mod metadata_extraction;

pub static AFTER_HELP: &str = cstr!("
<bold>Usage Instructions</bold>

Detailed usage information provided in long-form help page, available by <cyan>stencila help publish zenodo</cyan>.
");

pub static AFTER_LONG_HELP: &str = cstr!("
<bold>Further information</bold>

<i>Authentication</i>

To deposit a document at Zenodo, you must first have an authentication token that has the <blue>deposit:actions</> scope enabled.

To create an authentication token, log into Zenodo, visit your account's dashboard, then click <bold>Applications</>, followed by <bold>+ New Token</bold> within the <bold>Personal access tokens</>  section. Give the token a name and enable the <blue>deposit:actions</> the scope. Enable the <blue>deposit:write</> scope to enable the <cyan>--force</> flag.

To inform Stencila about the new token, add it as the STENCILA_ZENODO_TOKEN environment variable or include it as the <cyan>--token</> <green><<TOKEN>></> option.

<i>Recommended workflow</i>

We recommend starting with the Zenodo Sandbox at <<https://sandbox.zenodo.org/>>.

    <dim>$</> export STENCILA_ZENODO_TOKEN=\"<green><<TOKEN>></>\" <dim># from https://sandbox.zenodo.org/</>
    <dim>$</> stencila publish zenodo <green><<DOCUMENT_PATH>></>
    <dim>🎉 Draft deposition submitted</>
    <dim>🌐 URL: https://sandbox.zenodo.org/deposit/<i><<deposit-id>></> (visit to check details and publish)</>
    <dim>📑 DOI: 10.5282/zenodo.<i><<deposit-id>></></>
    <dim>Note: This deposit has been submitted to the Zenodo Sandbox.</>
    <dim>Note: Use the --zenodo flag to resubmit to the production Zenodo server.</>

You should now review the deposit, make any corrections and then click publish from Zenodo's web interface when you're happy. If you wish to skip the review process and publish immediately, then use the <cyan>--force</> flag.

Now that you have an understanding of the process, you should move to the Zenodo production server at <<https://zenodo.org/>>. This involves creating an authentication token there, informing Stencila about it and then adding the <cyan>--zenodo</> flag as a command-line argument.

    <dim>$</> export STENCILA_ZENODO_TOKEN=\"<green><<TOKEN>></>\" <dim># from https://zenodo.org/</>
    <dim>$</> stencila publish zenodo <bold>--zenodo</> <green><<DOCUMENT_PATH>></>
    <dim>🎉 Draft deposition submitted</>
    <dim>🌐 URL: https://zenodo.org/deposit/<i><<deposit-id>></> (visit to check details and publish)</>
    <dim>📑 DOI: 10.5281/zenodo.<i><<deposit-id>></></>

<i>Metadata</i>

Metadata for the deposition is provided by command-line arguments, falling back to metadata found within the document, then Stencila's defaults.

Zenodo requires that deposits have metadata such as <blue>title</> and <blue>description</>. It also requires that you describe which resource type and/or publication type the deposit is.

By default, Stencila describes your document as a publication, with the <i>preprint</i> sub-type. You can use the <cyan>--lesson</> flag to describe your document as a lesson. To use another publication sub-type, review the list in the documentation above and provide it as the <cyan>--publication=[<green><<PUBLICATION_TYPE>></>]</> option.

Every source format has its own mechanism for providing metadata. For example, within Stencila Markdown (.smd files), you add YAML front matter:

  <dim>---</>
  <cyan>title:</> <dim>Example Stencila Markdown</>
  <cyan>description:</> <dim>An example of a Stencila Markdown document with embedded metadata</>
  <dim>---</>
");

fn parse_date(input: &str) -> Result<schema::Date> {
    schema::Date::from_str(input)
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
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    #[arg(hide_env_values(true))]
    token: Option<String>,

    // Server selection options
    /// Publish to the Zenodo Sandbox for testing
    ///
    /// The Zenodo Sandbox is available at https://sandbox.zenodo.org. It
    /// requires its own access key that is independent from the Zenodo
    /// production server.
    ///
    /// [default]
    #[arg(group = "zenodo_server")]
    #[arg(long, default_value_t = true)]
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    #[arg(conflicts_with("zenodo"))]
    #[arg(default_value_ifs([
        ("zenodo", ArgPredicate::Equals("sandbox.zenodo.org".into()), "true"),
        ("zenodo", ArgPredicate::IsPresent, "false"),
    ]))]
    sandbox: bool,

    /// Specify Zenodo instance, defaults to the public-facing production server
    ///
    /// Use this option to publish to a custom Zenodo instance. Provide just the
    /// domain name or IP address with an optional port, e.g.
    /// `zenodo.example.org` or `zenodo.example.org:8000`.
    #[arg(group = "zenodo_server")]
    #[arg(long, value_parser = parse_host)]
    #[arg(help_heading("Zenodo Settings"), display_order(1))]
    #[arg(num_args(0..=1), require_equals=true, default_missing_value("zenodo.org"))]
    #[arg(default_value("zenodo.org"))] // This isn't actually used, but is useful for auto-generated documentation.
    #[arg(default_value_if("sandbox", ArgPredicate::IsPresent, "sandbox.zenodo.org"))]
    zenodo: url::Host,

    // Resource type options
    /// Upload document as a "lesson"
    #[arg(group = "resource_type")]
    #[arg(long)]
    #[arg(conflicts_with_all = ["is_publication", "publication"])]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    lesson: bool,

    /// Upload document as a "publication"
    ///
    /// [default]
    #[arg(group = "resource_type")]
    #[arg(long, default_value_t = true)]
    #[arg(default_value_if("lesson", ArgPredicate::IsPresent, "false"))]
    // #[arg(help_heading("Deposition Settings"), display_order(2))]
    #[arg(hide(true))] // needed for logic later, but not exposed to the user
    is_publication: bool,

    /// Reserve a DOI for the deposition
    #[arg(long)]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    reserve_doi: bool,

    /// Publication date
    ///
    /// Provide the date formatted as YYYY-MM-DD, e.g. 2012-03-10.
    ///
    /// When omitted, Zenodo will use today's date.
    #[arg(long, value_name = "YYYY-MM-DD")]
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

    /// Upload document as a "publication"
    ///
    /// Provide one of the publication types from Zenodo's controlled vocabulary.
    #[arg(long, value_name = "PUBLICATION_TYPE")]
    #[arg(default_value("preprint"))]
    #[arg(num_args(0..=1), require_equals=true)]
    #[arg(default_value_ifs([
        ("lesson", ArgPredicate::Equals("true".into()), None),
        ("publication", ArgPredicate::IsPresent, Some("preprint")),
    ]))]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    publication: Option<PublicationType>,

    /// Publish the deposition immediately
    ///
    /// Requires that access token provided by the `--token` option has the "deposit:write" scope.
    ///
    /// WARNING: This is permanent. It will be impossible to review the deposition or make changes
    ///          to it before it is publicly viewable. Publication cannot be revoked.
    #[arg(long)]
    #[arg(help("Publish the deposition immediately (use with care"))]
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
        .map(codec_text::to_text)
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
        let token = self
            .token
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
                // skip anything that isn't an article
                let Node::Article(article) = root else {
                    return None;
                };

                // return the first DOI in the article's identifiers
                if let Some(ids) = &article.options.identifiers {
                    for id in ids.iter() {
                        if let Some(doi) = metadata_extraction::extract_doi(id) {
                            return Some(doi.to_string());
                        }
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

        // Saving occurs in a second block to avoid holding a lock while calling await
        let save_doc = match doc_updated.lock() {
            Ok(update) if *update => true,
            Ok(_) => false,
            Err(_) => {
                tracing::error!("internal error: failed to acquire write access to file. Proceeding without modifying document.");
                false
            }
        };
        if save_doc {
            doc.save(CommandWait::Yes).await?
        }

        // Determine server URL
        let server_url = if self.sandbox {
            url::Host::parse("sandbox.zenodo.org")?
        } else {
            self.zenodo
        };

        let metadata_from_doc = doc.inspect(|root| {
            let Node::Article(article) = root else { return None };

            let title = article.title.as_ref().map(codec_text::to_text).or_else(|| self.title.clone()).unwrap_or_else(|| { "Untitled".to_string() });
            let description = article.description.as_ref().map(codec_text::to_text);
            let mut creators = Vec::new();

            if let Some(authors) = &article.authors {

                for author in authors {
                    if let schema::Author::Person(person) = author {
                                            let mut affiliation = None;

                                            // Zenodo expects ORCIDs to be numbers and hyphens only, e.g,
                                            // 0000-0000-0000-0000, although it the last digit can be a trailing
                                            // X to indicate a checksum
                                            let mut orcid = None;

                                            let name = metadata_extraction::extract_name(person);

                                            // find orcid in list of identifiers
                                            if let Some(ids) = person.options.identifiers.as_ref() {
                                                for id in ids {
                                                    orcid = metadata_extraction::extract_orcid(id);

                                                    if orcid.is_some() {
                                                        break
                                                    }
                                                }
                                            };

                                            if let Some(mut affs) = metadata_extraction::extract_affiliations(person) {
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
        }

        if self.is_publication {
            deposit["metadata"]["upload_type"] = json!("publication");
            deposit["metadata"]["publication_type"] = json!(self.publication.unwrap_or_default());
        }

        match (doi, self.reserve_doi) {
            (Some(doi), true) => tracing::warn!(
                "Document already has a DOI ({doi}). The --reserve-doi flag will be ignored."
            ),
            (Some(doi), false) => deposit["metadata"]["doi"] = json!(doi),
            (None, true) => deposit["metadata"]["prereserve_doi"] = json!(true),
            (None, false) => (),
        }

        tracing::debug!(metadata_provided = ?deposit, "Creating deposit");

        // Create HTTP client
        let client = Client::new();

        // Create deposition
        let deposition_response = client
            .post(format!("https://{server_url}/api/deposit/depositions"))
            .bearer_auth(token)
            .json(&deposit)
            .send()
            .await?;

        tracing::debug!(response = ?deposition_response, "Deposit creation response");

        // permissions error
        if deposition_response.status().as_u16() == 403 {
            let mut msg = String::from(
                "hint: Check that the access token is correct and has the necessary scope",
            );

            if self.force {
                msg.push_str("s (`deposit:actions` and `deposit:write`) enabled.");
            } else {
                msg.push_str(" (`deposit:actions`) enabled.");
            }

            tracing::info!(msg);
        }

        if !&deposition_response.status().is_success() {
            let data: Value = deposition_response.json().await?;

            if let Some(Value::Array(errors)) = data.get("errors") {
                for error in errors {
                    if let (Some(Value::String(field)), Some(Value::Array(messages))) =
                        (error.get("field"), error.get("messages"))
                    {
                        if field == "metadata.description" {
                            for message in messages.iter().filter_map(|msg| msg.as_str()) {
                                if message == "Field may not be null." {
                                    tracing::error!("Description missing from article.");
                                    if std::io::stdout().is_terminal() {
                                        cli_utils::message!("hint: Provide a description with the --description flag.");
                                    }
                                }
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
        let deposition_url = deposition["links"]["self"]
            .as_str()
            .ok_or_eyre("No deposition URL provided in the response from Zenodo")?;

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
            tracing::info!(
                "Dry run complete - bundle created at {}",
                swb_path.display()
            );
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
            .put(format!("{}/{}", bucket_url, file_name))
            .bearer_auth(token)
            .body(tokio::fs::read(&swb_path).await?)
            .send()
            .await?;

        if !upload_response.status().is_success() {
            tracing::error!(response = ?upload_response, file = ?file_name, "Failed to upload");
            bail!("Failed to upload file: {}", upload_response.text().await?);
        }

        let deposition_url = deposition["links"]["html"]
            .as_str()
            .ok_or_eyre("No deposit URL provided by Zenodo.")?;

        if self.force {
            // Publish the deposition
            let publish_response = client
                .post(format!(
                    "{}/api/deposit/depositions/{}/actions/publish",
                    server_url, deposition_id
                ))
                .bearer_auth(token)
                .send()
                .await?;

            if !publish_response.status().is_success() {
                bail!(
                    "Failed to publish deposition: {}",
                    publish_response.text().await?
                );
            }

            cli_utils::message!("🎉 Deposition published").to_stdout();
            cli_utils::message!("🌐 URL: {}", deposition_url).to_stdout();
            // TODO: supply DOI
        } else {
            cli_utils::message!("🎉 Draft deposition submitted").to_stdout();
            cli_utils::message!(
                "🌐 URL: {} (visit to check details and publish)",
                deposition_url
            )
            .to_stdout();

            if let Some(doi) = reserved_doi {
                cli_utils::message!("📑 DOI: {}", doi).to_stdout();
            }

            if self.sandbox {
                cli_utils::message!("Note: This deposit has been submitted to the Zenodo Sandbox.")
                    .to_stdout();
                cli_utils::message!(
                    "Note: Use the --zenodo flag to resubmit to the production Zenodo server."
                )
                .to_stdout();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn defaults_to_sandbox() {
        let args = Cli::parse_from(["publish-zenodo"]);
        assert!(args.sandbox);
    }

    #[test]
    fn publication_flag_unnecessary() {
        let args = Cli::parse_from(["publish-zenodo", "some.smd"]);
        assert!(matches!(args.publication, Some(PublicationType::Preprint)));
    }

    // TODO: check if there is a bug to file with clap (this works in practice but not under test conditions)
    //
    // #[test]
    // fn publication_type_defaults_to_preprint() {
    //     let args = Cli::parse_from(["publish-zenodo", "--publication", "some.smd"]);
    //     assert!(matches!(args.publication, Some(PublicationType::Preprint)));
    // }

    #[test]
    fn publication_type_can_be_specified() {
        let args = Cli::parse_from(["publish-zenodo", "--publication=report", "some.smd"]);
        assert!(args.sandbox);
        assert!(matches!(args.publication, Some(PublicationType::Report)));
    }

    #[test]
    fn upload_type_lesson_does_not_require_publication_type() {
        let args = Cli::parse_from(["publish-zenodo", "--lesson", "some.smd"]);
        assert!(args.lesson);
        assert!(args.sandbox);
        assert!(args.publication.is_none());
        assert!(!args.is_publication);
    }

    #[test]
    fn production_server_enabled_with_zenodo_flag() {
        let args = Cli::parse_from(["publish-zenodo", "--zenodo", "some.smd"]);
        assert_eq!(
            &format!("{}", args.zenodo),
            "zenodo.org",
            "--zenodo parsed as {}",
            args.zenodo
        );
        assert!(!args.sandbox, "CLI {args:#?}");
        assert!(matches!(args.publication, Some(PublicationType::Preprint)));
    }

    #[test]
    fn sandbox_flag_is_triggered_if_manually_specified() {
        let args = Cli::parse_from(["publish-zenodo", "--zenodo=sandbox.zenodo.org", "some.smd"]);
        assert_eq!(
            &format!("{}", args.zenodo),
            "sandbox.zenodo.org",
            "--zenodo parsed as {}",
            args.zenodo
        );
        assert!(args.sandbox, "CLI {args:#?}");
    }
}
