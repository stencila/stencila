use cli_utils::{
    table::{self, Attribute, Cell, Color},
    Code, ToStdout,
};
use codecs::{EncodeOptions, Format};
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
    },
    schema::{InstructionMessage, InstructionType, Node, Prompt, StringOrNumber},
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
    Select(Select),
    Update(Update),
    Reset(Reset),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List {}.run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Show(show) => show.run().await?,
            Command::Select(select) => select.run().await?,
            Command::Update(update) => update.run().await?,
            Command::Reset(update) => update.run().await?,
        }

        Ok(())
    }
}

/// List the prompts available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header(["Id", "Version", "Description"]);

        for prompt in super::list().await {
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
                Some(InstructionType::New) => Color::Green,
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
        let prompt = super::get(&self.id, &InstructionType::New).await?;

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

/// Select a prompt
///
/// Useful for checking which prompt will be matched to a given instruction
#[derive(Debug, Args)]
struct Select {
    /// The type of instruction
    r#type: InstructionType,

    /// The instruction message
    message: String,
}

impl Select {
    async fn run(self) -> Result<()> {
        let prompt = super::select(
            &self.r#type,
            &Some(InstructionMessage::from(self.message)),
            &None,
            &None,
        )
        .await?;

        println!("{}", prompt.id.clone().unwrap_or_default());

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
/// Reinitializes the builtin prompts directory to those prompts
/// embedded in this version of Stencila
#[derive(Debug, Args)]
struct Reset {}

impl Reset {
    async fn run(self) -> Result<()> {
        super::reset_builtin().await
    }
}
