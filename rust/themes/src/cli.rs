use clap::{Args, Parser, Subcommand};
use eyre::Result;
use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    stencila_format::Format,
    tabulated::{Attribute, Cell, Color, Tabulated},
};

use crate::{ThemeType, TokenScope, get, list, list_builtin_tokens, new, remove};

/// Manage themes
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available themes</dim>
  <b>stencila themes</b>

  <dim># Show the default resolved theme</dim>
  <b>stencila themes show</b>

  <dim># Show a specific theme</dim>
  <b>stencila themes show</b> <g>tufte</g>

  <dim># Create a new workspace theme</dim>
  <b>stencila themes new</b>

  <dim># Create a named user theme</dim>
  <b>stencila themes new</b> <g>my-theme</g>

  <dim># Remove a user theme</dim>
  <b>stencila themes remove</b> <g>my-theme</g>

  <dim># List design tokens available to use in themes</dim>
  <b>stencila themes tokens</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    New(New),
    Remove(Remove),
    Tokens(Tokens),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List.run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Show(show) => show.run().await,
            Command::Tokens(tokens) => tokens.run().await,
            Command::New(new) => new.run().await,
            Command::Remove(remove) => remove.run().await,
        }
    }
}

/// List the available themes
#[derive(Debug, Default, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List;

impl List {
    async fn run(self) -> Result<()> {
        let themes = list(None).await?;

        let mut table = Tabulated::new();
        table.set_header(["Name", "Type", "Location"]);

        for theme in themes {
            let name = theme.name.unwrap_or_else(|| "(workspace)".to_string());
            let location = theme.location.unwrap_or_default();

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold),
                match theme.r#type {
                    ThemeType::Workspace => Cell::new("workspace").fg(Color::Yellow),
                    ThemeType::User => Cell::new("user").fg(Color::Cyan),
                    ThemeType::Builtin => Cell::new("builtin").fg(Color::Green),
                },
                Cell::new(location),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available themes</dim>
  <b>stencila themes list</b>
"
);

/// Show the resolved theme CSS
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the theme to show
    ///
    /// If not provided, shows the default resolved theme following the
    /// resolution order: workspace theme.css → user default.css → builtin stencila.css
    name: Option<String>,

    /// Show resolved CSS variables
    #[arg(long, short)]
    verbose: bool,
}

impl Show {
    async fn run(self) -> Result<()> {
        match get(self.name.clone(), None).await? {
            Some(theme) => {
                Code::new(Format::Css, &theme.content).to_stdout();

                if self.verbose {
                    let mut vars = "/* Resolved CSS variables */\n\n".to_string();
                    for (name, value) in &theme.variables {
                        vars.push_str(&format!("--{} = {}\n", name, value));
                    }
                    Code::new(Format::Css, &vars).to_stdout();
                }

                Ok(())
            }
            None => {
                let name = self.name.as_deref().unwrap_or("default");
                message!("🔍 Theme `{}` not found", name);
                Ok(())
            }
        }
    }
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show the default resolved theme</dim>
  <b>stencila themes show</b>

  <dim># Show a specific theme by name</dim>
  <b>stencila themes show</b> <g>tufte</g>

  <dim># Show a user theme</dim>
  <b>stencila themes show</b> <g>my-theme</g>

  <dim># Show theme with resolved CSS variables</dim>
  <b>stencila themes show</b> <g>stencila</g> <c>--verbose</c>
"
);

/// Create a new theme
#[derive(Debug, Args)]
#[command(after_long_help = NEW_AFTER_LONG_HELP)]
struct New {
    /// The name of the theme to create
    ///
    /// If not provided, creates `theme.css` in the current directory.
    /// Otherwise, creates in the themes config directory.
    name: Option<String>,

    /// Overwrite the theme file if it already exists
    #[arg(long, short)]
    force: bool,
}

impl New {
    async fn run(self) -> Result<()> {
        if let Some(path) = new(self.name, self.force).await? {
            message!("🎨 Created theme at `{}`", path.display());
        }

        Ok(())
    }
}

pub static NEW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new workspace theme in the current folder</dim>
  <b>stencila themes new</b>

  <dim># Create a named user theme in the config folder</dim>
  <b>stencila themes new</b> <g>my-theme</g>

  <dim># Force overwrite an existing user theme</dim>
  <b>stencila themes new</b> <g>my-theme</g> <c>--force</c>
"
);

/// Remove a user theme
#[derive(Debug, Args)]
#[command(alias = "rm", after_long_help = REMOVE_AFTER_LONG_HELP)]
struct Remove {
    /// The name of the theme to remove
    name: String,

    /// Remove the theme without confirmation
    #[arg(long, short)]
    force: bool,
}

impl Remove {
    async fn run(self) -> Result<()> {
        remove(&self.name, self.force).await?;

        message!("🗑️ Removed theme `{}`", self.name);

        Ok(())
    }
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove a user theme</dim>
  <b>stencila themes remove</b> <g>my-theme</g>

  <dim># Force remove without confirmation</dim>
  <b>stencila themes remove</b> <g>my-theme</g> <c>--force</c>

  <dim># Use the rm alias</dim>
  <b>stencila themes rm</b> <g>my-theme</g>
"
);

/// List builtin theme tokens
#[derive(Debug, Default, Args)]
#[command(after_long_help = TOKENS_LIST_AFTER_LONG_HELP)]
struct Tokens {
    /// Filter by token scope
    #[arg(long)]
    scope: Option<TokenScope>,

    /// Filter by token family
    #[arg(long)]
    family: Option<String>,

    /// Output as a machine-readable format
    #[arg(long, value_name = "FORMAT")]
    r#as: Option<AsFormat>,
}

impl Tokens {
    async fn run(self) -> Result<()> {
        let scope = self.scope.map(Into::into);
        let tokens = list_builtin_tokens(scope, self.family.as_deref());

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &tokens)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Scope", "Family", "Default"]);

        for token in tokens {
            table.add_row([
                Cell::new(format!("--{}", token.name)).add_attribute(Attribute::Bold),
                Cell::new(token.scope.to_string()).fg(Color::Cyan),
                Cell::new(&token.family).fg(Color::Green),
                Cell::new(&token.default_value),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

pub static TOKENS_LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all builtin theme tokens</dim>
  <b>stencila themes tokens list</b>

  <dim># List tokens for a family</dim>
  <b>stencila themes tokens list</b> <c>--family</c> <g>admonition</g>

  <dim># List tokens for a scope</dim>
  <b>stencila themes tokens list</b> <c>--scope</c> <g>site</g>

  <dim># Output JSON for scripts and agents</dim>
  <b>stencila themes tokens list</b> <c>--family</c> <g>plot</g> <c>--as</c> <g>json</g>
"
);
