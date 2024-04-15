use assistant::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
        serde_yaml,
    },
    context::Context,
    format::Format,
    schema::{InstructionBlock, InstructionMessage},
    AssistantAvailability, AssistantType, GenerateOptions,
};
use cli_utils::{
    table::{self, Attribute, Cell, CellAlignment, Color},
    Code, ToStdout,
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
        table.set_header([
            "Name",
            "Provider",
            "Version",
            "Context len.",
            "Inputs",
            "Outputs",
        ]);

        for assistant in super::list().await {
            use AssistantAvailability::*;
            let availability = assistant.availability();

            let inputs = assistant
                .supported_inputs()
                .iter()
                .map(|input| input.to_string())
                .join(", ");

            let outputs = assistant
                .supported_outputs()
                .iter()
                .map(|output| output.to_string())
                .join(", ");

            table.add_row([
                Cell::new(assistant.name()).add_attribute(Attribute::Bold),
                match assistant.r#type() {
                    AssistantType::Builtin => Cell::new("builtin").fg(Color::Green),
                    AssistantType::Local => Cell::new("local").fg(Color::Cyan),
                    AssistantType::Remote => Cell::new("remote").fg(Color::Magenta),
                    AssistantType::Plugin(name) => {
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
                match assistant.context_length() {
                    0 => Cell::new(""),
                    _ => Cell::new(assistant.context_length()).set_alignment(CellAlignment::Right),
                },
                Cell::new(inputs),
                Cell::new(outputs),
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
    /// The id of the assistant to execute the instruction
    ///
    /// For example, `stencila/insert-code-chunk` or `mistral/mistral-medium`.
    /// For Stencila assistants, the org prefix can be omitted e.g. `insert-code-chunk`.
    /// See `stencila assistants list` for a list of available assistants.
    id: String,

    /// The instruction to execute
    instruction: String,
}

impl Execute {
    async fn run(self) -> Result<()> {
        let mut instruction =
            InstructionBlock::new(vec![InstructionMessage::from(&self.instruction)]);
        instruction.assignee = Some(self.id);

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
