use ask::{ask_with_default, Answer};
use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::Result,
};

/// Uninstall the Stencila CLI
pub fn uninstall() -> Result<()> {
    self_replace::self_delete()?;

    Ok(())
}

/// Uninstall this command line tool
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># Uninstall Stencila CLI (with confirmation prompt)</dim>
  <blue>></blue> stencila uninstall

<bold><blue>Note</blue></bold>
  This will permanently remove the Stencila CLI binary from your system.
  Your documents and data will not be affected, only the CLI tool itself.
  You can reinstall Stencila at any time from https://stencila.io or GitHub.
"
);

impl Cli {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        if !ask_with_default(
            "Are you sure you want to uninstall Stencila CLI?",
            Answer::Yes,
        )
        .await?
        .is_yes()
        {
            return Ok(());
        }

        uninstall()?;
        eprintln!("😢 Successfully uninstalled");

        Ok(())
    }
}
