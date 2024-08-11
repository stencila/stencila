use cli_utils::{
    table::{self, Attribute, Cell},
    Code, ToStdout,
};
use codecs::{EncodeOptions, LossesResponse};
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
        InstructionMessage, InstructionModel, InstructionType, Node, StringOrNumber, Thing,
        Timestamp,
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
            "Id",
            "Name",
            "Version",
            "Instructions",
            "Node types",
            "Description",
        ]);

        for assistant in super::list().await {
            let Assistant {
                id,
                name,
                version,
                instruction_types,
                node_types,
                description,
                ..
            } = assistant.inner;

            let version = match version {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };

            table.add_row([
                Cell::new(id.unwrap_or_default()).add_attribute(Attribute::Bold),
                Cell::new(name),
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
    message: String,

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

    /// The output format for the suggestion
    #[arg(long, short, default_value = "md")]
    to: Format,
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
                self.message,
                Some(vec![Author::AuthorRole(instructor.clone())]),
            )),
            assignee: self.assignee,
            model: Some(Box::new(InstructionModel {
                id_pattern: self.name_pattern,
                minimum_score: self.minimum_score,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!("Instruction");
        Code::new(Format::Yaml, &serde_yaml::to_string(&instruction)?).to_stdout();

        let assistant = find(
            &instruction.instruction_type,
            &instruction.message,
            &instruction.assignee,
            &None,
        )
        .await?;

        println!("Assistant");
        println!("{}\n", assistant.id.as_deref().unwrap_or_default());

        let prompter = AuthorRole {
            last_modified: Some(Timestamp::now()),
            ..assistant.clone().into()
        };

        let system_message = render(assistant).await?;

        println!("Assistant prompt (no context)");
        Code::new(Format::Markdown, &system_message).to_stdout();

        let suggestion =
            execute_instruction_block(vec![instructor], prompter, &system_message, &instruction)
                .await?;

        println!("Suggestion");
        let output = codecs::to_string(
            &Node::SuggestionBlock(suggestion),
            Some(EncodeOptions {
                format: Some(self.to.clone()),
                losses: LossesResponse::Debug,
                ..Default::default()
            }),
        )
        .await?;
        Code::new(self.to, &output).to_stdout();

        Ok(())
    }
}
