use clap::{Parser, Subcommand};
use cli_utils::color_print::cstr;
use eyre::Result;

use crate::{
    check::Check, disable::Disable, enable::Enable, install::Install, link::Link, list::List,
    show::Show, uninstall::Uninstall,
};

/// Manage plugins
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Install(Install),
    Uninstall(Uninstall),
    Link(Link),
    Enable(Enable),
    Disable(Disable),
    Show(Show),
    Check(Check),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(args) => args.run().await?,
            Command::Install(args) => args.run().await?,
            Command::Uninstall(args) => args.run().await?,
            Command::Link(args) => args.run().await?,
            Command::Enable(args) => args.run().await?,
            Command::Disable(args) => args.run().await?,
            Command::Show(args) => args.run().await?,
            Command::Check(args) => args.run().await?,
        }

        Ok(())
    }
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available plugins</dim>
  <b>stencila plugins</>

  <dim># Install a plugin from a URL</dim>
  <b>stencila plugins install</> <g>https://github.com/user/plugin.git</>

  <dim># Install a plugin from a local directory</dim>
  <b>stencila plugins install</> <g>./my-plugin</>

  <dim># Show details about a plugin</dim>
  <b>stencila plugins show</> <g>my-plugin</>

  <dim># Enable a plugin</dim>
  <b>stencila plugins enable</> <g>my-plugin</>

  <dim># Disable a plugin</dim>
  <b>stencila plugins disable</> <g>my-plugin</>

  <dim># Check plugin health</dim>
  <b>stencila plugins check</> <g>my-plugin</>

  <dim># Uninstall a plugin</dim>
  <b>stencila plugins uninstall</> <g>my-plugin</>

<bold><b>Plugin Management</b></bold>
  Plugins can extend Stencila's functionality by adding support for
  new formats, kernels, models, and other features.
"
);
