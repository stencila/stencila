use cli_utils::{
    table::{self, Attribute, Cell, Color},
    AsFormat, Code, ToStdout,
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
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

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
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

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
            let (Ok(from_codec), Ok(to_codec)) = (
                super::get(None, Some(&format), Some(CodecDirection::Decode)),
                super::get(None, Some(&format), Some(CodecDirection::Encode)),
            ) else {
                continue;
            };

            formats.push(FormatSpecification {
                name: format.name().into(),
                extension: format.extension(),
                lossless: format.is_lossless(),
                from: from_codec.availability(),
                to: to_codec.availability(),
            })
        }
        formats.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &formats)?.to_stdout();

            return Ok(());
        }

        let mut table = table::new();
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
