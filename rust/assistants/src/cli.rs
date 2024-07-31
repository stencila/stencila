use cli_utils::{
    table::{self, Attribute, Cell},
    Code, ToStdout,
};
use common::itertools::Itertools;
use model::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        serde_yaml,
    },
    format::Format,
    schema::{
        Assistant, Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, InstructionBlock,
        InstructionMessage, InstructionModel, InstructionType, StringOrNumber, Thing, Timestamp,
    },
};

use crate::{execute_instruction_block, find, render};

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
            "Version",
            "Instructions",
            "Node types",
            "Description",
        ]);

        for Assistant {
            name,
            version,
            instruction_types,
            node_types,
            description,
            ..
        } in super::list().await
        {
            let version = match version {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold),
                Cell::new(version),
                Cell::new(
                    instruction_types
                        .iter()
                        .map(|typ| typ.to_string())
                        .join(", "),
                ),
                Cell::new(node_types.join(", ")),
                Cell::new(description.as_str()),
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
    /// The text of the instruction
    instruction: String,

    /// The name of the assistant assigned to the instruction
    ///
    /// For example, `stencila/paragraph` or `my-org/abstract`.
    /// For Stencila assistants, the org prefix can be omitted e.g. `insert-code-chunk`.
    /// See `stencila assistants list` for a list of available assistants.
    #[arg(long, short)]
    assignee: Option<String>,

    /// The regex pattern to filter model names by
    #[arg(long, short = 'm')]
    name_pattern: Option<String>,

    /// The threshold score for selecting a model to use
    #[arg(long, short = 'y')]
    minimum_score: Option<u64>,
}

impl Execute {
    async fn run(self) -> Result<()> {
        let instructor = AuthorRole {
            role_name: AuthorRoleName::Instructor,
            author: AuthorRoleAuthor::Thing(Thing::default()),
            last_modified: Some(Timestamp::now()),
            ..Default::default()
        };

        let instruction = InstructionBlock {
            instruction_type: InstructionType::New,
            message: Some(InstructionMessage::user(
                self.instruction,
                Some(vec![Author::AuthorRole(instructor.clone())]),
            )),
            assignee: self.assignee,
            model: Some(Box::new(InstructionModel {
                name_pattern: self.name_pattern,
                minimum_score: self.minimum_score,
                ..Default::default()
            })),
            ..Default::default()
        };

        let assistant = find(&instruction.assignee, &instruction.instruction_type, &None).await?;

        let prompter = AuthorRole {
            last_modified: Some(Timestamp::now()),
            ..assistant.clone().into()
        };

        let system_message = render(assistant).await?;

        let suggestion =
            execute_instruction_block(instructor, prompter, &system_message, &instruction).await?;

        println!("Instruction");
        Code::new(Format::Yaml, &serde_yaml::to_string(&instruction)?).to_stdout();

        println!("System prompt (no context)");
        Code::new(Format::Markdown, &system_message).to_stdout();

        println!("Suggestion");
        Code::new(Format::Yaml, &serde_yaml::to_string(&suggestion)?).to_stdout();

        Ok(())
    }
}
