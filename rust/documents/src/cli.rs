use std::{path::PathBuf, str::FromStr, sync::Arc};

use cli_utils::{
    args::params,
    clap::{self, Parser},
    result,
    table::{Table, Title},
    Result, Run,
};
use common::{
    async_trait::async_trait,
    eyre::{self},
    itertools::Itertools,
    serde::Serialize,
    serde_json,
    tokio::sync::Mutex,
    tracing,
};
use graph::{PlanOptions, PlanOrdering};
use graph_triples::resources;
use node_address::Address;
use node_patch::{diff, diff_display};
use stencila_schema::{
    EnumValidator, IntegerValidator, Node, NumberValidator, StringValidator, ValidatorTypes,
};

use crate::{document::Document, messages::When};

use super::*;

/// Manage documents
#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Parser)]
pub enum Action {
    List(List),
    Open(Open),
    Close(Close),
    Show(Show),

    #[cfg(feature = "kernels-cli")]
    Execute(kernel_commands::Execute),
    #[cfg(feature = "kernels-cli")]
    Kernels(kernel_commands::Kernels),
    #[cfg(feature = "kernels-cli")]
    Tasks(kernel_commands::Tasks),
    #[cfg(feature = "kernels-cli")]
    Cancel(kernel_commands::Cancel),
    #[cfg(feature = "kernels-cli")]
    Symbols(kernel_commands::Symbols),
    #[cfg(feature = "kernels-cli")]
    Restart(kernel_commands::Restart),

    Graph(Graph),
    #[clap(alias = "pars")]
    Params(Params),
    Run(Run_),
    Plan(Plan),
    Query(Query),
    Diff(Diff),
    Merge(Merge),
    Detect(Detect),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        let Self { action } = self;
        match action {
            Action::List(action) => action.run().await,
            Action::Open(action) => action.run().await,
            Action::Close(action) => action.run().await,
            Action::Show(action) => action.run().await,

            #[cfg(feature = "kernels-cli")]
            Action::Execute(action) => action.run().await,
            #[cfg(feature = "kernels-cli")]
            Action::Kernels(action) => action.run().await,
            #[cfg(feature = "kernels-cli")]
            Action::Tasks(action) => action.run().await,
            #[cfg(feature = "kernels-cli")]
            Action::Cancel(action) => action.run().await,
            #[cfg(feature = "kernels-cli")]
            Action::Symbols(action) => action.run().await,
            #[cfg(feature = "kernels-cli")]
            Action::Restart(action) => action.run().await,

            Action::Graph(action) => action.run().await,
            Action::Params(action) => action.run().await,
            Action::Run(action) => action.run().await,
            Action::Plan(action) => action.run().await,
            Action::Query(action) => action.run().await,
            Action::Diff(action) => action.run().await,
            Action::Merge(action) => action.run().await,
            Action::Detect(action) => action.run().await,
        }
    }
}

// The arguments used to specify the document file path and format
// Reused (with flatten) below
#[derive(Parser)]
struct File {
    /// The path of the document file
    path: String,

    /// The format of the document file
    #[clap(short, long)]
    format: Option<String>,
}
impl File {
    async fn open(&self) -> eyre::Result<String> {
        DOCUMENTS.open(&self.path, self.format.clone()).await
    }

    async fn get(&self) -> eyre::Result<Arc<Mutex<Document>>> {
        let id = self.open().await?;
        DOCUMENTS.get(&id).await
    }
}

/// List open documents
#[derive(Parser)]
pub struct List {}
#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        let list = DOCUMENTS.list().await?;
        result::value(list)
    }
}

/// Open a document
#[derive(Parser)]
pub struct Open {
    #[clap(flatten)]
    file: File,
}
#[async_trait]
impl Run for Open {
    async fn run(&self) -> Result {
        self.file.open().await?;
        result::nothing()
    }
}

/// Close a document
#[derive(Parser)]
pub struct Close {
    /// The path of the document file
    pub path: String,
}
#[async_trait]
impl Run for Close {
    async fn run(&self) -> Result {
        DOCUMENTS.close(&self.path).await?;
        result::nothing()
    }
}

/// Show a document
#[derive(Parser)]
pub struct Show {
    #[clap(flatten)]
    file: File,

    /// A pointer to the part of the document to show e.g. `variables`, `format.name`
    ///
    /// Some, usually large, document properties are only shown when specified with a
    /// pointer (e.g. `content` and `root`).
    pub pointer: Option<String>,
}
#[async_trait]
impl Run for Show {
    async fn run(&self) -> Result {
        let document = Document::open(&self.file.path, self.file.format.clone()).await?;
        if let Some(pointer) = &self.pointer {
            if pointer == "content" {
                result::content(&document.format.extension, &document.content)
            } else if pointer == "root" {
                let root = &*document.root.read().await;
                result::value(root)
            } else {
                todo!()
                //let data = serde_json::to_value(document)?;
                //if let Some(part) = data.pointer(&json::pointer(pointer)) {
                //    Ok(result::value(part)?)
                //} else {
                //    bail!("Invalid pointer for document: {}", pointer)
                //}
            }
        } else {
            result::value(document)
        }
    }
}

// Subcommands that only work if `kernels-cli` feature is enabled
#[cfg(feature = "kernels-cli")]
mod kernel_commands {
    use super::*;

    #[derive(Parser)]
    #[clap(alias = "exec")]
    pub struct Execute {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        execute: kernels::commands::Execute,
    }

    #[async_trait]
    impl Run for Execute {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let mut kernels = document.kernels.write().await;
            self.execute.run(&mut kernels).await?;
            result::nothing()
        }
    }

    #[derive(Parser)]
    pub struct Kernels {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        kernels: kernels::commands::Running,
    }

    #[async_trait]
    impl Run for Kernels {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.read().await;
            self.kernels.run(&*kernels).await
        }
    }

    #[derive(Parser)]
    pub struct Tasks {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        tasks: kernels::commands::Tasks,
    }

    #[async_trait]
    impl Run for Tasks {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.read().await;
            self.tasks.run(&*kernels).await
        }
    }

    #[derive(Parser)]
    pub struct Cancel {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        cancel: kernels::commands::Cancel,
    }

    #[async_trait]
    impl Run for Cancel {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let mut kernels = document.kernels.write().await;
            self.cancel.run(&mut *kernels).await?;
            result::nothing()
        }
    }

    #[derive(Parser)]
    pub struct Symbols {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        symbols: kernels::commands::Symbols,
    }

    #[async_trait]
    impl Run for Symbols {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.read().await;
            self.symbols.run(&*kernels).await
        }
    }

    #[derive(Parser)]
    pub struct Restart {
        #[clap(flatten)]
        file: File,

        #[clap(flatten)]
        restart: kernels::commands::Restart,
    }

    #[async_trait]
    impl Run for Restart {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.read().await;
            self.restart.run(&*kernels).await
        }
    }
}

/// Output the dependency graph for a document
///
/// Tip: When using the DOT format (the default), if you have GraphViz and ImageMagick
/// installed you can view the graph by piping the output to them. For example, to
/// view a graph of the current project:
///
/// ```sh
/// $ stencila documents graph | dot -Tpng | display
/// ```
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct Graph {
    #[clap(flatten)]
    file: File,

    /// The format to output the graph as
    #[clap(long, short, default_value = "dot", possible_values = &graph::FORMATS)]
    to: String,
}

#[async_trait]
impl Run for Graph {
    async fn run(&self) -> Result {
        let document = self.file.get().await?;
        let document = document.lock().await;
        let content = document.graph.read().await.to_format(&self.to)?;
        result::content(&self.to, &content)
    }
}

/// Show the parameters of a document
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct Params {
    #[clap(flatten)]
    file: File,
}

/// A row in the table of parameters
#[derive(Serialize, Table)]
#[serde(crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
struct Param {
    #[table(title = "Name")]
    name: String,

    #[table(title = "Id")]
    id: String,

    #[table(skip)]
    address: Address,

    #[table(title = "Validation", display_fn = "option_validator")]
    validator: Option<ValidatorTypes>,

    #[table(title = "Default", display_fn = "option_node")]
    default: Option<Node>,
}

fn option_validator(validator: &Option<ValidatorTypes>) -> String {
    let validator = match validator {
        Some(validator) => validator,
        None => return String::new(),
    };
    match validator {
        ValidatorTypes::BooleanValidator(..) => "Boolean".to_string(),
        ValidatorTypes::NumberValidator(NumberValidator {
            minimum,
            maximum,
            multiple_of,
            ..
        }) => format!(
            "Number {} {} {}",
            minimum
                .map(|min| format!("min:{}", min))
                .unwrap_or_default(),
            maximum
                .map(|max| format!("max:{}", max))
                .unwrap_or_default(),
            multiple_of
                .as_ref()
                .map(|mult| format!("multiple-of:{}", mult))
                .unwrap_or_default()
        )
        .trim()
        .to_string(),
        ValidatorTypes::IntegerValidator(IntegerValidator {
            minimum,
            maximum,
            multiple_of,
            ..
        }) => format!(
            "Integer {} {} {}",
            minimum
                .map(|min| format!("min:{}", min))
                .unwrap_or_default(),
            maximum
                .map(|max| format!("max:{}", max))
                .unwrap_or_default(),
            multiple_of
                .as_ref()
                .map(|mult| format!("multiple-of:{}", mult))
                .unwrap_or_default()
        )
        .trim()
        .to_string(),
        ValidatorTypes::StringValidator(StringValidator {
            min_length,
            max_length,
            pattern,
            ..
        }) => format!(
            "String {} {} {}",
            min_length
                .map(|min| format!("min-length:{}", min))
                .unwrap_or_default(),
            max_length
                .map(|max| format!("max-length:{}", max))
                .unwrap_or_default(),
            pattern
                .as_ref()
                .map(|pattern| format!("pattern:{}", pattern))
                .unwrap_or_default()
        )
        .trim()
        .to_string(),
        ValidatorTypes::EnumValidator(EnumValidator { values, .. }) => format!(
            "One of {}",
            values
                .iter()
                .map(|value| serde_json::to_string(value).unwrap_or_default())
                .join(", ")
        )
        .trim()
        .to_string(),
        _ => "*other*".to_string(),
    }
}

fn option_node(validator: &Option<Node>) -> String {
    let node = match validator {
        Some(node) => node,
        None => return String::new(),
    };
    serde_json::to_string(node).unwrap_or_default()
}

#[async_trait]
impl Run for Params {
    async fn run(&self) -> Result {
        let document = self.file.get().await?;
        let mut document = document.lock().await;
        let params = document.params().await?;
        let params = params
            .into_iter()
            .map(|(name, (id, address, param))| Param {
                name,
                id,
                address,
                validator: param.validator.map(|boxed| *boxed),
                default: param.default.map(|boxed| *boxed),
            })
            .collect_vec();
        result::table(params, Param::title())
    }
}

/// Run a document
#[derive(Parser)]
pub struct Run_ {
    /// The path of the document to execute
    pub input: PathBuf,

    /// Parameter `name=value` pairs
    args: Vec<String>,

    /// The path to save the executed document
    #[clap(short, long, alias = "out")]
    output: Option<PathBuf>,

    /// The format of the input (defaults to being inferred from the file extension or content type)
    #[clap(short, long)]
    from: Option<String>,

    /// The format of the output (defaults to being inferred from the file extension)
    #[clap(short, long)]
    to: Option<String>,

    /// The theme to apply to the output (only for HTML and PDF)
    #[clap(short = 'e', long)]
    theme: Option<String>,

    /// The id of the node to start execution from
    #[clap(short, long)]
    start: Option<String>,

    /// Ordering for the execution plan
    #[clap(long, parse(try_from_str = PlanOrdering::from_str), ignore_case = true)]
    ordering: Option<PlanOrdering>,

    /// Maximum concurrency for the execution plan
    ///
    /// A maximum concurrency of 2 means that no more than two tasks will
    /// run at the same time (ie. in the same stage).
    /// Defaults to the number of CPUs on the machine.
    #[clap(short, long)]
    concurrency: Option<usize>,
}

#[async_trait]
impl Run for Run_ {
    async fn run(&self) -> Result {
        // Open document
        let mut document = Document::open(&self.input, self.from.clone()).await?;

        // Call with args, or just execute
        if !self.args.is_empty() {
            let args = params(&self.args);
            document.call(args).await?;
        } else {
            document
                .execute(
                    When::Never,
                    self.start.clone(),
                    self.ordering,
                    self.concurrency,
                )
                .await?;
        }

        tracing::info!("Finished running document");

        // Display or write output
        if let Some(output) = &self.output {
            let out = output.display().to_string();
            if out == "-" {
                let format = self.to.clone().unwrap_or_else(|| "json".to_string());
                let content = document.dump(Some(format.clone()), None).await?;
                return result::content(&format, &content);
            } else {
                document
                    .write_as(output, self.to.clone(), self.theme.clone())
                    .await?;
            }
        }

        result::nothing()
    }
}

/// Generate an execution plan for a document
#[derive(Parser)]
pub struct Plan {
    /// The path of the document to execute
    pub input: PathBuf,

    /// The format of the input (defaults to being inferred from the file extension or content type)
    #[clap(short, long)]
    from: Option<String>,

    /// The id of the node to start execution from
    #[clap(short, long)]
    start: Option<String>,

    /// Ordering for the execution plan
    #[clap(short, long, parse(try_from_str = PlanOrdering::from_str), ignore_case = true)]
    ordering: Option<PlanOrdering>,

    /// Maximum concurrency for the execution plan
    ///
    /// A maximum concurrency of 2 means that no more than two tasks will
    /// run at the same time (ie. in the same stage).
    /// Defaults to the number of CPUs on the machine.
    #[clap(short, long)]
    concurrency: Option<usize>,
}

#[async_trait]
impl Run for Plan {
    async fn run(&self) -> Result {
        // Open document
        let document = Document::open(&self.input, self.from.clone()).await?;

        let start = self
            .start
            .as_ref()
            .map(|node_id| resources::code(&document.path, node_id, "", None));

        let options = PlanOptions {
            ordering: self.ordering.unwrap_or_else(PlanOptions::default_ordering),
            max_concurrency: self
                .concurrency
                .unwrap_or_else(PlanOptions::default_max_concurrency),
        };

        let plan = {
            let graph = document.graph.write().await;
            graph.plan(start, None, None, Some(options)).await?
        };

        result::new("md", &plan.to_markdown(), &plan)
    }
}

/// Query a document
#[derive(Parser)]
pub struct Query {
    /// The path of the document file
    file: String,

    /// The query to run on the document
    query: String,

    /// The format of the file
    #[clap(short, long)]
    format: Option<String>,

    /// The language of the query
    #[clap(
            short,
            long,
            default_value = "jmespath",
            possible_values = &node_query::LANGS
        )]
    lang: String,
}

#[async_trait]
impl Run for Query {
    async fn run(&self) -> Result {
        let Self {
            file,
            format,
            query,
            lang,
        } = self;
        let document_id = DOCUMENTS.open(file, format.clone()).await?;
        let document = DOCUMENTS.get(&document_id).await?;
        let document = document.lock().await;
        let node = &*document.root.read().await;
        let result = node_query::query(node, query, Some(lang))?;
        result::value(result)
    }
}

/// Display the structural differences between two documents
#[derive(Parser)]
pub struct Diff {
    /// The path of the first document
    first: PathBuf,

    /// The path of the second document
    second: PathBuf,

    /// The format to display the difference in
    ///
    /// Defaults to a "unified diff" of the JSON representation
    /// of the documents. Unified diffs of other formats are available
    /// e.g. "md", "yaml". Use "raw" for the raw patch as a list of
    /// operations.
    #[clap(short, long, default_value = "json")]
    format: String,
}

#[async_trait]
impl Run for Diff {
    async fn run(&self) -> Result {
        let Self {
            first,
            second,
            format,
        } = self;
        let first = Document::open(first, None).await?;
        let second = Document::open(second, None).await?;

        let first = &*first.root.read().await;
        let second = &*second.root.read().await;

        if format == "raw" {
            let patch = diff(first, second);
            result::value(patch)
        } else {
            let diff = diff_display(first, second, format).await?;
            result::content("patch", &diff)
        }
    }
}

/// Merge changes from two or more derived versions of a document
///
/// This command can be used as a Git custom "merge driver".
/// First, register Stencila as a merge driver,
///
/// ```sh
/// $ git config merge.stencila.driver "stencila merge --git %O %A %B"
/// ```
///
/// (The placeholders `%A` etc are used by `git` to pass arguments such
/// as file paths and options to `stencila`.)
///
/// Then, in your `.gitattributes` file assign the driver to specific
/// types of files e.g.,
///
/// ```text
/// *.{md|docx} merge=stencila
/// ```
///
/// This can be done per project, or globally.
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
// See https://git-scm.com/docs/gitattributes#_defining_a_custom_merge_driver and
// https://www.julianburr.de/til/custom-git-merge-drivers/ for more examples of defining a
// custom driver. In particular the meaning of the placeholders %O, %A etc
pub struct Merge {
    /// The path of the original version
    original: PathBuf,

    /// The paths of the derived versions
    #[clap(required = true, multiple_occurrences = true)]
    derived: Vec<PathBuf>,

    /// A flag to indicate that the command is being used as a Git merge driver
    ///
    /// When the `merge` command is used as a Git merge driver the second path
    /// supplied is the file that is written to.
    #[clap(short, long)]
    git: bool,
}

#[async_trait]
impl Run for Merge {
    async fn run(&self) -> Result {
        let mut original = Document::open(&self.original, None).await?;

        let mut docs: Vec<Document> = Vec::new();
        for path in &self.derived {
            docs.push(Document::open(path, None).await?)
        }

        original.merge(&docs).await?;

        if self.git {
            original.write_as(&self.derived[0], None, None).await?;
        } else {
            original.write(None, None).await?;
        }

        result::nothing()
    }
}

/// Detect entities within a document
#[derive(Parser)]
pub struct Detect {
    /// The path of the document file
    pub file: String,
}

#[async_trait]
impl Run for Detect {
    async fn run(&self) -> Result {
        let mut document = Document::open(&self.file, None).await?;
        document.read(true).await?;
        let nodes = document.detect().await?;
        result::value(nodes)
    }
}
