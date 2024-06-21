use cli_utils::{
    table::{self, Attribute, Cell, Color},
    Code, ToStdout,
};
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        serde_yaml,
    },
    context::Context,
    format::Format,
    schema::{InstructionBlock, InstructionMessage, InstructionType},
    GenerateOptions, ModelAvailability, ModelType,
};

use crate::execute_instruction;

/// Manage assistants
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Execute(Execute),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List {}.run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Execute(execute) => execute.run().await?,
        }

        Ok(())
    }
}

/// List the assistant available
#[derive(Debug, Args)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let mut table = table::new();
        table.set_header(["Name", "Provider", "Version", "Description"]);

        for assistant in super::list(true).await {
            use ModelAvailability::*;
            let availability = assistant.availability();

            table.add_row([
                Cell::new(assistant.name()).add_attribute(Attribute::Bold),
                match assistant.r#type() {
                    ModelType::Builtin => Cell::new("builtin").fg(Color::Green),
                    ModelType::Local => Cell::new("local").fg(Color::Cyan),
                    ModelType::Remote => Cell::new("remote").fg(Color::Magenta),
                    ModelType::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::Blue)
                    }
                },
                match availability {
                    Available => Cell::new(assistant.version()),
                    _ => match availability {
                        Available => Cell::new(availability).fg(Color::Green),
                        Disabled => Cell::new(availability).fg(Color::DarkBlue),
                        Installable => Cell::new(availability).fg(Color::Cyan),
                        Unavailable => Cell::new(availability).fg(Color::Grey),
                    },
                },
                Cell::new(assistant.description().unwrap_or_default()),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}

/// Execute an instruction with an assistant
///
/// Mainly intended for quick testing of assistants during development.
#[derive(Debug, Args)]
#[clap(alias = "exec")]
struct Execute {
    /// The name of the assistant to execute the instruction
    ///
    /// For example, `stencila/insert-code-chunk` or `mistral/mistral-medium`.
    /// For Stencila assistants, the org prefix can be omitted e.g. `insert-code-chunk`.
    /// See `stencila assistants list` for a list of available assistants.
    name: String,

    /// The instruction to execute
    instruction: String,
}

impl Execute {
    async fn run(self) -> Result<()> {
        let mut instruction = InstructionBlock::new(
            InstructionType::New,
            vec![InstructionMessage::user(self.instruction)],
        );
        instruction.assignee = Some(self.name);

        let context = Context::default();
        let options = GenerateOptions::default();
        let output = execute_instruction(instruction.clone(), context, options).await?;

        println!("Instruction");
        Code::new(Format::Yaml, &serde_yaml::to_string(&instruction)?).to_stdout();

        println!("Output");
        Code::new(Format::Yaml, &serde_yaml::to_string(&output)?).to_stdout();

        Ok(())
    }
}
