// TODO: use the [Research Organization Registry (ROR)](https://ror.org/) database to gather affiliations for authors

use std::{path::PathBuf, str::FromStr};

use cli_utils::{cli_hint, hint, message, parse_host, ToStdout};

use codec::schema::ConfigPublishZenodoAccessRight;
use common::{
    clap::{
        self,
        builder::{ArgPredicate, PossibleValue},
        Parser, ValueHint,
    },
    eyre::{bail, OptionExt, Result},
    reqwest::Client,
    serde,
    serde_json::{json, Value},
    tokio, tracing,
};

use color_print::cstr;
use document::{schema, schema::Node, CommandWait, Document};

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
    #[arg(value_hint(ValueHint::FilePath))]
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

    /// Reserve a DOI for the deposition (overrides DOI in Article metadata, if any)
    #[arg(long)]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    #[arg(conflicts_with = "doi")]
    reserve_doi: bool,

    /// Supply an existing DOI
    ///
    /// Use this field to provide a DOI that has already been issued
    /// for the material you are depositing.
    #[arg(long)]
    #[arg(help_heading("Deposition Settings"), display_order(2))]
    #[arg(value_parser = metadata_extraction::parse_doi)]
    doi: Option<String>,

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
    /// HTML is allowed.
    #[arg(long)]
    #[arg(help("Description notes (HTML permitted)"))]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    description: Option<String>,

    /// License Identifier (examples: cc-by, cc0)
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(default_value_if("access_right", "restricted", None))]
    #[arg(default_value_if("access_right", "open", Some("cc-by")))]
    #[arg(required_if_eq("access_right", "embargoed"))]
    license: Option<String>,

    /// Closed Access
    ///
    /// Public access of the deposition is not allowed.
    ///
    /// Shorthand for `--access-right=closed`.
    #[arg(group = "access")]
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    closed: bool,

    /// Set `--access-right` to restricted
    #[arg(group = "access")]
    #[arg(alias = "restrict")]
    #[arg(long, value_name = "ACCESS_CONDITIONS")]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    restricted: bool,

    /// Provide a date when the embargo ends
    #[arg(group = "access")]
    #[arg(alias = "embargo")]
    #[arg(alias = "embargo_date")]
    #[arg(long, value_name = "YYYY-MM-DD")]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(value_parser = parse_date)]
    embargoed: Option<schema::Date>,

    /// Conditions to fulfill to access deposition
    ///
    /// Describe the conditions of access to the deposition for
    /// be accessed when --access-right=restricted. HTML is allowed.
    #[arg(long)]
    #[arg(help = "Conditions to fulfill to access deposition (HTML permitted)")]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(required_if_eq("access_right", "restricted"))]
    #[arg(required_if_eq("restricted", "true"))]
    access_conditions: Option<String>,

    /// Access right
    ///
    ///
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(group = "access")]
    #[arg(alias = "access")]
    #[arg(value_parser([
        PossibleValue::new("open").help("Open Access. Sets the default license to CC-BY, e.g. --license='cc-by'."),
        PossibleValue::new("embargoed").help("Embargoed Access. Requires --access_conditions, --license, and --embargoed=<DATE>."),
        PossibleValue::new("restricted").help("Restricted Access. Requires --access_conditions."),
        PossibleValue::new("closed").help("Closed Access.")
    ]))]
    #[arg(default_value_ifs = [
        ("restricted", ArgPredicate::Equals("true".into()), "restricted"),
        ("embargoed", ArgPredicate::IsPresent, "embargoed"),
        ("closed", ArgPredicate::Equals("true".into()), "closed"),
    ])]
    #[arg(default_value = "open")]
    access_right: String,

    /// Comma-delimited list of keywords
    ///
    /// To add multiple keywords, separate them with commas: --keywords=testing,software
    ///
    /// To include spaces in keywords, surround the list with quotes[*]: --keywords='testing,software,software testing'   
    ///
    /// [*] The exact syntax will depend on your shell language.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(require_equals(true), value_delimiter(','))]
    keywords: Vec<String>,

    /// Methodology
    ///
    /// Free-form description of the methodology used in this research.
    /// HTML is allowed.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(help("Methodology (HTML permitted)"))]
    method: Option<String>,

    /// Additional Notes
    ///
    /// Any additional notes that to do not fit within the description.
    /// HTML is allowed.
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    #[arg(help("Additional notes (HTML permitted)"))]
    notes: Option<String>,

    /// Version of document
    ///
    /// NOTE: this is a free text field and all inputs are be accepted. However,
    /// the suggested format is a semantically versioned tag (see more details
    /// on semantic versioning at semver.org).
    #[arg(long)]
    #[arg(help_heading("Deposition Metadata"), display_order(3))]
    version: Option<String>,

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
        let Ok(doc) = Document::open(&self.path).await else {
            tracing::error!("Document root is not an Article");
            hint!("Attempt to re-render a standalone document and retry.").to_stdout();
            cli_hint!(
                "If you built the file with `stencila render`, try adding the `--standalone` flag."
            )
            .to_stdout();
            bail!(
                "Unable to create a Document from file at {}",
                self.path.display()
            );
        };
        doc.compile(CommandWait::Yes).await?;

        // Pre-check: ensure that we have an Article
        let Node::Article(_) = &doc.root().await else {
            tracing::info!("Internal error: Document root is not an Article");
            bail!("Document root is not an Article");
        };

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
            let mut doi = None;

            if let Some(ids) = &article.options.identifiers {
                for id in ids.iter() {
                    if doi.is_none() {
                        doi = metadata_extraction::extract_doi(id).map(|x| x.to_string())
                    }
                }
            }

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

            // Config YAML header
            let mut embargoed = None;
            let mut access_right = None;
            let mut notes = None;
            let mut method = None;

            tracing::debug!("article:{:?}",article);
            if let Some(config) = &article.config {
                tracing::debug!("config:{:?}",config);
                if let Some(publish) = &config.publish {
                    if let Some(publisher) = &publish.zenodo{
                        embargoed = publisher.embargoed.clone();
                        access_right = publisher.access_right.clone();
                        notes = publisher.notes.clone();
                        method = publisher.method.clone();
                    }
                }
            }

            Some((title, description, creators, doi, embargoed, access_right, notes, method))
        }).await;

        let mut doi_from_doc = None;
        let mut deposit = json!({ "metadata": json!({}) });

        if let Some((title, description, creators, doi, embargoed, access_right, notes, method)) =
            metadata_from_doc.clone()
        {
            deposit["metadata"]["title"] = json!(title);
            deposit["metadata"]["description"] = json!(description);
            deposit["metadata"]["creators"] = json!(creators);
            doi_from_doc = doi;

            tracing::debug!("{:?}", embargoed);
            tracing::debug!("{:?}", access_right);
            if let Some(schema::Date {
                value: embargo_date,
                ..
            }) = embargoed
            {
                debug_assert_eq!(
                    access_right,
                    Some(ConfigPublishZenodoAccessRight::Embargoed),
                    "logic error: --embargoed={:?} is set, but --access_right={:?}",
                    embargo_date,
                    access_right
                );
                if access_right != Some(ConfigPublishZenodoAccessRight::Embargoed) {
                    message!("Note: An embargo date ({}) has been provided, but access right is set to {:?}. Replacing access right to `embargoed`.", embargo_date, access_right);
                }
                deposit["metadata"]["embargo_date"] = json!(embargo_date);
                deposit["metadata"]["access_right"] = json!("embargoed");
            }

            tracing::debug!("{:?}", notes);
            deposit["metadata"]["notes"] = json!(notes);
            tracing::debug!("{:?}", method);
            deposit["metadata"]["method"] = json!(method);
        }

        if let Some(title_from_args) = self.title {
            deposit["metadata"]["title"] = json!(title_from_args);
        }

        if let Some(desc) = self.description {
            deposit["metadata"]["description"] = json!(desc);
        }

        if let Some(date) = self.publication_date {
            deposit["metadata"]["publication_date"] = json!(date);
        }

        if !self.keywords.is_empty() {
            let kw: Vec<_> = self
                .keywords
                .into_iter()
                .map(|kw| kw.trim().to_string())
                .collect();
            deposit["metadata"]["keywords"] = json!(kw);
        }

        if let Some(method) = self.method {
            deposit["metadata"]["method"] = json!(method);
        }

        if let Some(notes) = self.notes {
            deposit["metadata"]["notes"] = json!(notes);
        }

        if let Some(ver) = self.version {
            deposit["metadata"]["version"] = json!(ver);
        }

        if self.lesson {
            deposit["metadata"]["upload_type"] = json!("lesson");
        } else if self.is_publication || self.publication.is_some() {
            deposit["metadata"]["upload_type"] = json!("publication");
            deposit["metadata"]["publication_type"] = json!(self.publication.unwrap_or_default());
        } else {
            bail!("Publication type unavailable")
        }

        let mut doi = None;
        let mut reserve_doi = false;

        if self.reserve_doi {
            match (&self.doi, &doi_from_doc) {
                (None, None) => reserve_doi = true,
                (None, Some(from_doc)) => {
                    tracing::debug!("Requesting a new DOI, although one ({from_doc}) was found within the Article metadata");
                    deposit["metadata"]["prereserve_doi"] = json!(true);
                    doi = Some(from_doc);
                }
                (Some(from_cli), Some(_)) | (Some(from_cli), None) => {
                    // these pattern should be impossible as --doi and --reserve-doi are marked as conflicting
                    if cfg!(debug_assertions) {
                        panic!("should be impossible to reserve a DOI and also provide one");
                    }
                    message!(
                        "Using DOI provided ({}), rather than pre-reserving another one.",
                        from_cli
                    )
                    .to_stdout();
                    doi = Some(from_cli)
                }
            }
        } else {
            match (&self.doi, &doi_from_doc) {
                (None, None) => tracing::debug!("No DOI was explicitly requested, nor was one provided. Relying on Zenodo's defaults."),
                (None, Some(from_doc)) => {
                    tracing::info!("Providing DOI found in Article metadata ({from_doc}).");
                    doi = Some(from_doc);
                },
                (Some(from_cli), None) => doi = Some(from_cli),
                (Some(from_cli), Some(from_doc)) if from_cli == from_doc => doi = Some(from_cli),
                (Some(from_cli), Some(from_doc)) => {
                    tracing::debug!("DOI provided ({from_cli}) does not match the DOI found within the Article metadata ({from_doc}). Preferring {from_cli}.");
                    doi = Some(from_cli);
                },
            }
        };

        if cfg!(debug_assertions) && reserve_doi && doi.is_some() {
            tracing::warn!("logic error - --reserve_doi and --doi permitted together");
        }

        if reserve_doi {
            deposit["metadata"]["prereserve_doi"] = json!(true)
        }

        if let Some(doi) = doi {
            deposit["metadata"]["doi"] = json!(doi)
        }

        tracing::debug!("{:?}",metadata_from_doc);
        if let Some(metadata) = metadata_from_doc.clone() {
            if let Some(access_right) = metadata.5 {
                tracing::debug!("access_right:{:?}",access_right);
                deposit["metadata"]["access_right"] = json!(access_right.to_string().to_lowercase().clone());
            } else {
                deposit["metadata"]["access_right"] = json!(self.access_right.clone());
            }
        } else {
            deposit["metadata"]["access_right"] = json!(self.access_right.clone());
        }
        //if let Some((..,license,..)) = metadata_from_doc.clone(){
        deposit["metadata"]["license"] = json!(self.license);
        //}

        if cfg!(debug_assertions) {
            if let Some(license) = self.license {
                if license == "cc-by" || license == "cc0" {
                    debug_assert_eq!(self.access_right, "open");
                }
            }
        }

        if let Some(schema::Date {
            value: embargo_date,
            ..
        }) = self.embargoed
        {
            debug_assert_eq!(
                &self.access_right, "embargoed",
                "logic error: --embargoed={:?} is set, but --access_right={}",
                embargo_date, self.access_right
            );
            if &self.access_right != "embargoed" {
                message!("Note: An embargo date ({}) has been provided, but access right is set to {}. Replacing access right to `embargoed`.", embargo_date, self.access_right);
            }
            deposit["metadata"]["embargo_date"] = json!(embargo_date);
            deposit["metadata"]["access_right"] = json!("embargoed");
        }

        if self.restricted {
            debug_assert_eq!(
                &self.access_right, "restricted",
                "logic error: --restricted is set, but --access_right={}",
                self.access_right
            );
            debug_assert!(
                self.access_conditions.is_none(),
                "logic error: --restricted is set, but --access_conditions has not been provided"
            );
            deposit["metadata"]["access_right"] = json!("restricted");
        }

        if self.closed {
            debug_assert_eq!(
                &self.access_right, "closed",
                "logic error: --closed is set, but --access_right={}",
                self.access_right
            );
            deposit["metadata"]["access_right"] = json!("closed");
        }

        if let Some(conditions) = self.access_conditions {
            deposit["metadata"]["access_conditions"] = json!(conditions);
        }

        tracing::debug!(metadata_provided = ?deposit, "Creating deposit");

        if self.dry_run {
            message!("🏁🏃‍♀️ Dry run completed").to_stdout();
            return Ok(());
        }

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
            let mut msg =
                String::from("Check that the access token is correct and has the necessary scope");
            if self.force {
                msg.push_str("s (`deposit:actions` and `deposit:write`) enabled.");
            } else {
                msg.push_str(" (`deposit:actions`) enabled.");
            }
            hint!("{}", msg).to_stdout();
            hint!("Check that the access token is provided by the Zenodo server that you're uploading to ({})", server_url).to_stdout();
        }

        if !&deposition_response.status().is_success() {
            let http_code = deposition_response.status().as_u16();
            let data: Value = deposition_response.json().await?;
            let debug_info = common::serde_json::to_string_pretty(&data)?;

            if let Some(Value::String(top_level_message)) = data.get("message") {
                tracing::info!(
                    error_from_zenodo = debug_info,
                    message_from_zenodo = top_level_message,
                    http_code = http_code,
                    "Failed to create deposition"
                );
            } else {
                tracing::info!(
                    error_from_zenodo = debug_info,
                    "Failed to create deposition"
                );
            }

            // TODO: use an actual type for deserialization rather than this maze of code
            if let Some(Value::Array(errors)) = data.get("errors") {
                for error in errors {
                    let Some(Value::String(field)) = error.get("field") else {
                        continue;
                    };
                    let Some(Value::Array(messages)) = error.get("messages") else {
                        continue;
                    };

                    match field.as_str() {
                        "metadata.description" => {
                            for message in messages.iter().filter_map(|msg| msg.as_str()) {
                                if message == "Field may not be null." {
                                    tracing::error!("Description missing from article.");
                                    cli_hint!("Provide a description with the --description flag.")
                                        .to_stdout();
                                } else {
                                    tracing::error!("Problem with description: {:?}", message)
                                }
                            }
                        }
                        "metadata.title" => {
                            for message in messages.iter().filter_map(|msg| msg.as_str()) {
                                if message == "Field may not be null." {
                                    tracing::error!("Title missing from article.");
                                    cli_hint!("Provide a title with the --title flag.").to_stdout();
                                } else {
                                    tracing::error!("Problem with title: {:?}", message)
                                }
                            }
                        }
                        "metadata.license" => {
                            for message in messages.iter().filter_map(|msg| msg.as_str()) {
                                if message.starts_with("Invalid license") {
                                    tracing::error!("Invalid license identifier provided.");
                                    hint!("Check that you are using an identifier, rather than the license's full name. Many identifiers are available in human-readable form at <https://spdx.org/licenses/>. Zenodo's full list of supported licenses is provided programmatically at <https://zenodo.org/api/vocabularies/licenses>.").to_stdout();
                                } else {
                                    tracing::error!("Problem with title: {:?}", message)
                                }
                            }
                        }
                        other_field => {
                            for message in messages.iter().filter_map(|msg| msg.as_str()) {
                                tracing::error!("Problem with `{other_field}` field: {message:?}")
                            }
                        }
                    }
                }
            }

            bail!("Failed to create deposition");
        }

        let deposition: Value = deposition_response.json().await?;
        let deposition_id = deposition["id"]
            .as_u64()
            .ok_or_eyre("No deposition ID in response in the response from Zenodo")?;

        let reserved_doi = deposition["metadata"]["prereserve_doi"]["doi"].as_str();
        let deposition_url = deposition["links"]["self"]
            .as_str()
            .ok_or_eyre("No deposition URL provided in the response from Zenodo")?;

        // Get bucket URL for file upload
        let bucket_url = deposition["links"]["bucket"]
            .as_str()
            .ok_or_eyre("No bucket URL in response")?;

        if let Some(doi) = reserved_doi {
            tracing::debug!(id = ?deposition_id, reserved_doi = doi, url = deposition_url, "Depositition created");
        }

        // Upload the SWB file using same filename
        let file_name = self
            .path
            .file_name()
            .ok_or_eyre("unable to infer file name from path")? // should never happen - we've already checked that it's a file
            .to_str()
            .ok_or_eyre("unable to convert file name to UTF-8")?;
        let upload_url: String = format!("{}/{}", bucket_url, file_name);
        tracing::debug!(
            url = upload_url,
            file = file_name,
            "Uploading file to deposit"
        );
        let upload_response = client
            .put(upload_url)
            .bearer_auth(token)
            .body(tokio::fs::read(&self.path).await?) // TODO: use streaming
            .send()
            .await?;

        if !upload_response.status().is_success() {
            tracing::error!(file = ?file_name, "Failed to upload {file_name}");
            bail!("Failed to upload file: {}", upload_response.text().await?);
        } else {
            tracing::debug!(file = ?file_name, "Upload successful");
        }

        let deposition_url = deposition["links"]["html"]
            .as_str()
            .ok_or_eyre("No deposit URL provided by Zenodo.")?;

        if self.force {
            let publish_url = format!(
                "{}/api/deposit/depositions/{}/actions/publish",
                server_url, deposition_id
            );
            tracing::debug!("Publishing deposit");
            // Publish the deposition
            let publish_response = client.post(publish_url).bearer_auth(token).send().await?;

            if !publish_response.status().is_success() {
                tracing::debug!(response = ?publish_response, "Details from HTTP response to publish deposition");

                // TODO: use diagnostics from publish-ghost
                bail!(
                    "Failed to publish deposition: {}",
                    publish_response.text().await?
                );
            }

            cli_utils::message!("🎉 Deposition published").to_stdout();
            cli_utils::message!("🌐 URL: {}", deposition_url).to_stdout();
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
