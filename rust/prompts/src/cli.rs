use cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use codecs::{EncodeOptions, Format};
use common::itertools::Itertools;
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
    },
    schema::{InstructionType, Node, Prompt, StringOrNumber},
};

/// Manage prompts
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available prompts</dim>
  <b>stencila prompts</>

  <dim># Show details about a specific prompt</dim>
  <b>stencila prompts show</> <g>edit-text</>

  <dim># Infer which prompt would be used for a query</dim>
  <b>stencila prompts infer</> <c>--instruction-type</> <g>create</> <y>\"Make a table\"</>

  <dim># Update builtin prompts from remote</dim>
  <b>stencila prompts update</>

  <dim># Reset prompts to embedded defaults</dim>
  <b>stencila prompts reset</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Infer(Infer),
    Update(Update),
    Reset(Reset),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Show(show) => show.run().await?,
            Command::Infer(infer) => infer.run().await?,
            Command::Update(update) => update.run().await?,
            Command::Reset(update) => update.run().await?,
        }

        Ok(())
    }
}

/// List the prompts available
///
/// Shows all available prompts with their names, descriptions, and versions.
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all prompts in table format</dim>
  <b>stencila prompts list</>

  <dim># Output prompts as JSON</dim>
  <b>stencila prompts list</> <c>--as</> <g>json</>
"
);

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list().await;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "Version"]);

        for prompt in list {
            let Prompt {
                name,
                description,
                version,
                instruction_types,
                ..
            } = prompt.inner;

            let version = match version {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };

            let color = match instruction_types.first() {
                Some(InstructionType::Discuss) => Color::Magenta,
                Some(InstructionType::Create) => Color::Green,
                Some(InstructionType::Describe) => Color::Blue,
                Some(InstructionType::TemplateDescribe) => Color::Blue, // Same as Describe
                Some(InstructionType::Edit) => Color::Cyan,
                Some(InstructionType::Fix) => Color::Yellow,
                None => Color::Grey,
            };

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold).fg(color),
                Cell::new(description.as_str()),
                Cell::new(version),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Show a prompt
///
/// Displays the full content and metadata of a specific prompt in the requested
/// format.
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the prompt to show
    name: String,

    /// The format to show the prompt in
    #[arg(long, short, default_value = "md")]
    to: Format,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show a prompt as Markdown</dim>
  <b>stencila prompts show</> <g>edit-text</>

  <dim># Show a prompt as JSON</dim>
  <b>stencila prompts show</> <g>create-table</> <c>--to</> <g>json</>
"
);

impl Show {
    async fn run(self) -> Result<()> {
        let prompt = super::get(&self.name).await?;

        let content = codecs::to_string(
            &Node::Prompt(prompt.inner),
            Some(EncodeOptions {
                format: Some(self.to.clone()),
                ..Default::default()
            }),
        )
        .await?;

        Code::new(self.to, &content).to_stdout();

        Ok(())
    }
}

/// Infer a prompt from a query
///
/// Useful for checking which prompt will be matched to a given instruction
/// type, node types, and/or query
#[derive(Debug, Args)]
#[command(after_long_help = INFER_AFTER_LONG_HELP)]
struct Infer {
    /// The instruction type
    #[arg(short, long)]
    instruction_type: Option<InstructionType>,

    /// The node types
    #[arg(short, long)]
    node_types: Option<String>,

    /// The query
    query: Option<String>,
}

pub static INFER_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Infer prompt with a specific query</dim>
  <b>stencila prompts infer</> <y>\"Update this paragraph based on latest data\"</>

  <dim># Infer for a specific instruction type</dim>
  <b>stencila prompts infer</> <c>--instruction-type</> <g>create</> <y>\"list of top regions\"</>
"
);

impl Infer {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let node_types = self
            .node_types
            .map(|value| value.split(",").map(String::from).collect_vec());

        match super::infer(&self.instruction_type, &node_types, &self.query.as_deref()).await {
            Some(prompt) => eprintln!("{}", prompt.name),
            None => eprintln!("Unable to infer prompt"),
        };

        Ok(())
    }
}

/// Update builtin prompts
///
/// Downloads the latest versions of builtin prompts from the Stencila prompts
/// repository. This adds new prompts and updates existing ones while preserving
/// any custom modifications you may have made.
#[derive(Debug, Args)]
#[command(after_long_help = UPDATE_AFTER_LONG_HELP)]
struct Update {}

pub static UPDATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Update builtin prompts from https://github.com/stencila/stencila</dim>
  <b>stencila prompts update</>
"
);

impl Update {
    async fn run(self) -> Result<()> {
        super::update_builtin().await
    }
}

/// Reset builtin prompts
///
/// Re-initializes the builtin prompts directory to those prompts
/// embedded in this version of Stencila
#[derive(Debug, Args)]
#[command(after_long_help = RESET_AFTER_LONG_HELP)]
struct Reset {}

pub static RESET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Reset prompts to embedded defaults</dim>
  <b>stencila prompts reset</>

<bold><b>Warning</b></bold>
  This will overwrite any custom modifications you have
  made to builtin prompts, restoring them to the versions
  embedded in this Stencila release.
"
);

impl Reset {
    async fn run(self) -> Result<()> {
        super::initialize_builtin(true).await?;
        Ok(())
    }
}
