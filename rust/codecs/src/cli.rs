use cli_utils::{
    color_print::cstr,
    tabulated::{Attribute, Cell, Color},
    AsFormat, Code, Tabulated, ToStdout,
};
use codec::{
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        serde::Serialize,
        strum::IntoEnumIterator,
    },
    format::Format,
    CodecAvailability, CodecDirection,
};

/// List the support for formats
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># List all supported formats</dim>
  <blue>></blue> stencila formats list

  <dim># Output formats as JSON</dim>
  <blue>></blue> stencila formats list --as json

<bold><blue>Format Support</blue></bold>
  • <blue>From</blue>: Whether the format can be read/imported
  • <blue>To</blue>: Whether the format can be written/exported
  • <blue>Lossless</blue>: Whether conversion preserves all data
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
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
    "<bold><blue>Examples</blue></bold>
  <dim># List all supported formats in table format</dim>
  <blue>></blue> stencila formats list

  <dim># Export format information as JSON</dim>
  <blue>></blue> stencila formats list --as json

<bold><blue>Columns</blue></bold>
  • <blue>Name</blue>: The format name
  • <blue>Extension</blue>: Default file extension
  • <blue>From</blue>: Can read/import this format
  • <blue>To</blue>: Can write/export this format
  • <blue>Lossless</blue>: Whether conversion preserves all data
"
);

/// Specifications for a format
#[derive(Serialize)]
#[serde(crate = "codec::common::serde", rename_all = "camelCase")]
pub struct FormatSpecification {
    name: String,
    extension: String,
    from: CodecAvailability,
    to: CodecAvailability,
    lossless: bool,
}

impl List {
    async fn run(self) -> Result<()> {
        let mut formats = Vec::new();
        for format in Format::iter() {
            let from = super::get(None, Some(&format), Some(CodecDirection::Decode)).map_or_else(
                |_| CodecAvailability::Unavailable,
                |codec| codec.availability(),
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
                lossless: format.is_lossless(),
                from,
                to,
            })
        }
        formats.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &formats)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Default Extension", "From", "To", "Lossless"]);

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

            table.add_row([
                Cell::new(format.name).add_attribute(Attribute::Bold),
                Cell::new(format.extension),
                from,
                to,
                lossless,
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}
