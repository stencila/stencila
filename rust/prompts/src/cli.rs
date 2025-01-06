use cli_utils::{
    table::{self, Attribute, Cell, Color},
    AsFormat, Code, ToStdout,
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
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

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
#[derive(Default, Debug, Args)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list().await;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = table::new();
        table.set_header(["Id", "Version", "Description"]);

        for prompt in list {
            let Prompt {
                id,
                version,
                description,
                instruction_types,
                ..
            } = prompt.inner;

            let version = match version {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };

            let color = match instruction_types.first() {
                Some(InstructionType::Create) => Color::Green,
                Some(InstructionType::Edit) => Color::Blue,
                Some(InstructionType::Fix) => Color::Cyan,
                Some(InstructionType::Describe) => Color::Yellow,
                None => Color::Grey,
            };

            table.add_row([
                Cell::new(id.unwrap_or_default())
                    .add_attribute(Attribute::Bold)
                    .fg(color),
                Cell::new(version),
                Cell::new(description.as_str()),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}

/// Show a prompt
#[derive(Debug, Args)]
struct Show {
    /// The id of the prompt to show
    id: String,

    /// The format to show the prompt in
    #[arg(long, short, default_value = "yaml")]
    to: Format,
}

impl Show {
    async fn run(self) -> Result<()> {
        let prompt = super::get(&self.id).await?;

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
/// Useful for checking which prompt will be matched to a given
/// instruction type, node types, and/or query
#[derive(Debug, Args)]
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

impl Infer {
    async fn run(self) -> Result<()> {
        let node_types = self
            .node_types
            .map(|value| value.split(",").map(String::from).collect_vec());

        match super::infer(&self.instruction_type, &node_types, &self.query.as_deref()).await {
            Some(prompt) => println!("{}", prompt.id.clone().unwrap_or_default()),
            None => println!("Unable to infer prompt"),
        };

        Ok(())
    }
}

/// Update builtin prompts
#[derive(Debug, Args)]
struct Update {}

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
struct Reset {}

impl Reset {
    async fn run(self) -> Result<()> {
        super::initialize_builtin(true).await?;
        Ok(())
    }
}
