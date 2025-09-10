use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use strum::{EnumMessage, IntoEnumIterator};

use stencila_cli_utils::{
    AsFormat, Code, Tabulated, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Color},
};
use stencila_codec::{
    CodecAvailability, CodecDirection, StructuringOperation, StructuringOptions, eyre::Result,
    stencila_format::Format,
};

/// List the support for formats
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold></bold>
  <dim># List all supported formats</dim>
  <b>stencila formats list</b>

  <dim># Output formats as JSON</dim>
  <b>stencila formats list</b> <c>--as</c> <g>json</g>

<bold><b>Format Support</b></bold>
  • <g>From</g>: Whether the format can be read/imported
  • <g>To</g>: Whether the format can be written/exported
  • <g>Lossless</g>: Whether conversion preserves all data
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Structuring(Structuring),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Structuring(structuring) => structuring.run().await?,
        }

        Ok(())
    }
}

/// List the support for formats
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold></bold>
  <dim># List all supported formats in table format</dim>
  <b>stencila formats list</b>

  <dim># Export format information as JSON</dim>
  <b>stencila formats list</b> <c>--as</c> <g>json</g>

<bold><b>Columns</b></bold>
  • <g>Name</g>: The format name
  • <g>Extension</g>: Default file extension
  • <g>From</g>: Can read/import this format
  • <g>To</g>: Can write/export this format
  • <g>Lossless</g>: Whether conversion preserves all data
"
);

/// Specifications for a format
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatSpecification {
    name: String,
    extension: String,
    from: CodecAvailability,
    to: CodecAvailability,
    lossless: bool,
    structuring_options: StructuringOptions,
}

impl List {
    async fn run(self) -> Result<()> {
        let mut formats = Vec::new();
        for format in Format::iter() {
            let (from, structuring_options) =
                super::get(None, Some(&format), Some(CodecDirection::Decode)).map_or_else(
                    |_| {
                        (
                            CodecAvailability::Unavailable,
                            StructuringOptions::default(),
                        )
                    },
                    |codec| (codec.availability(), codec.structuring_options(&format)),
                );

            let to = super::get(None, Some(&format), Some(CodecDirection::Encode)).map_or_else(
                |_| CodecAvailability::Unavailable,
                |codec| codec.availability(),
            );

            if matches!(from, CodecAvailability::Unavailable)
                && matches!(to, CodecAvailability::Unavailable)
            {
                continue;
            }

            formats.push(FormatSpecification {
                name: format.name().into(),
                extension: format.extension(),
                from,
                to,
                lossless: format.is_lossless(),
                structuring_options,
            })
        }
        formats.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &formats)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header([
            "Name",
            "Default Extension",
            "From",
            "To",
            "Lossless",
            "Default Structuring",
        ]);

        for format in formats {
            let from = match format.from {
                CodecAvailability::Available => Cell::new("yes").fg(Color::Green),
                CodecAvailability::Installable(package) => {
                    Cell::new(format!("requires {package}")).fg(Color::Yellow)
                }
                CodecAvailability::Unavailable => Cell::new("no").fg(Color::Red),
            };

            let to = match format.to {
                CodecAvailability::Available => Cell::new("yes").fg(Color::Green),
                CodecAvailability::Installable(package) => {
                    Cell::new(format!("requires {package}")).fg(Color::Yellow)
                }
                CodecAvailability::Unavailable => Cell::new("no").fg(Color::Red),
            };

            let lossless = if format.lossless {
                Cell::new("yes").fg(Color::Green)
            } else {
                Cell::new("no").fg(Color::Yellow)
            };

            let structuring = format.structuring_options.to_string();

            table.add_row([
                Cell::new(format.name).add_attribute(Attribute::Bold),
                Cell::new(format.extension),
                from,
                to,
                lossless,
                Cell::new(structuring),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Get a list of possible structuring operations
#[derive(Default, Debug, Args)]
#[command(after_long_help = STRUCTURING_AFTER_LONG_HELP)]
struct Structuring {
    /// Provide longer details on each structuring operation
    #[arg(long, short)]
    verbose: bool,
}

pub static STRUCTURING_AFTER_LONG_HELP: &str = cstr!(
    "<bold></bold>
  <dim># List all structuring operations</dim>
  <b>stencila formats structuring</b>

  <dim># List all structuring operations with details for each</dim>
  <b>stencila formats structuring --verbose</b>
"
);

impl Structuring {
    async fn run(self) -> Result<()> {
        let mut table = Tabulated::new();

        let mut header = vec!["Name", "Description"];
        if self.verbose {
            header.push("Details");
        }
        table.set_header(header);

        for op in StructuringOperation::iter() {
            let docs = op.get_documentation().unwrap_or_default();
            let mut parts = docs.split("\n\n");
            let desc = parts.next().unwrap_or_default();
            let details = parts.next().unwrap_or_default().replace("\n", " ");

            let mut row = vec![
                Cell::new(op.to_string()).add_attribute(Attribute::Bold),
                Cell::new(desc),
            ];
            if self.verbose {
                row.push(Cell::new(details));
            }
            table.add_row(row);
        }

        table.to_stdout();

        Ok(())
    }
}
