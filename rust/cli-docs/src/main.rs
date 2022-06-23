//! Generate documentation from the Stencila `clap` CLI application

use std::{
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
};

use cli_utils::{
    clap::{App, Args, Command},
    common::{eyre::Result, itertools::Itertools},
};

use stencila::cli::Cli;

fn main() -> Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference");
    create_dir_all(&dir)?;

    let app = Cli::augment_args(Command::new("cli"));
    let global_options = get_options(&app, true);
    render_app(&dir, &app, "stencila", &global_options)
}

fn render_app(dir: &Path, app: &App, command: &str, global_options: &str) -> Result<()> {
    let name = app.get_name();

    let header = get_header(app);

    let (title, intro) = get_title_intro(app);

    let usage = get_usage(app, command);

    let args = get_args(app);

    let options = if command == "stencila" {
        String::new()
    } else {
        get_options(app, false)
    };

    let (subcommands, path) = if app.get_subcommands().count() > 0 {
        let dir = dir.join(name);
        create_dir_all(&dir)?;

        let subcommands = render_subcommands(app, &dir, command, global_options)?;
        (subcommands, dir.join("index.md"))
    } else {
        (String::new(), dir.join(format!("{}.md", name)))
    };

    let md = format!(
        r"
{header}

<!-- Generated from doc comments in Rust. Do not edit. -->

{title}
{usage}
{intro}
{subcommands}
{args}
{options}
{global_options}
"
    );
    write(path, md.trim())?;

    Ok(())
}

fn get_header(app: &App) -> String {
    let parts = app
        .get_subcommands()
        .map(|sub| format!("  - {}", sub.get_name()))
        .join("\n");

    if parts.is_empty() {
        String::new()
    } else {
        format!("---\nparts:\n{}\n---\n", parts)
    }
}

fn get_title_intro(app: &App) -> (String, String) {
    let title = app.get_about().unwrap_or_default();

    let intro = app
        .get_long_about()
        .and_then(|long_about| long_about.strip_prefix(title))
        .map(|desc| desc.trim())
        .unwrap_or_default()
        .to_string()
        + "\n";

    let name = app.get_name();
    let name = if name == "cli" { "stencila" } else { name };

    let title = format!("# `{name}`: {title}\n");

    (title, intro)
}

fn get_usage(app: &App, command: &str) -> String {
    let mut usage = format!("{command} [options]");
    if app.get_subcommands().count() > 0 {
        usage += " <subcommand>"
    } else {
        for arg in app.get_positionals() {
            let name = arg.get_name();
            if arg.is_required_set() {
                usage += &format!(" <{}>", name);
            } else {
                usage += &format!(" [{}]", name);
            }
        }
    }
    format!("## Usage\n\n```sh\n{usage}\n```\n")
}

fn render_subcommands(
    app: &App,
    dir: &Path,
    command: &str,
    global_options: &str,
) -> Result<String> {
    let mut md = "## Subcommands\n\n".to_string();

    md += "| Name | Description |\n| --- | --- |\n";
    for subcommand in app.get_subcommands() {
        let name = subcommand.get_name();
        let title = subcommand.get_about().unwrap_or_default();
        md += &format!("| [`{name}`]({name}) | {title} |\n");

        render_app(
            dir,
            subcommand,
            &format!("{} {}", command, name),
            global_options,
        )?
    }
    md += "| `help` | Print help information |\n";

    Ok(md)
}

fn get_args(app: &App) -> String {
    let mut md = "## Arguments\n\n".to_string();

    md += "| Name | Description |\n| --- | --- |\n";
    let mut count = 0;
    for arg in app.get_positionals() {
        let name = arg.get_name();

        count += 1;

        md += &format!("| `{}` | {} |\n", name, arg.get_help().unwrap_or_default());
    }

    if count == 0 {
        String::new()
    } else {
        md
    }
}

fn get_options(app: &App, global: bool) -> String {
    let mut md = format!(
        "## {heading}\n\n",
        heading = if global { "Global options" } else { "Options" }
    );

    md += "| Name | Description |\n| --- | --- |\n";
    let mut count = 0;
    for option in app.get_arguments().filter(|arg| !arg.is_positional()) {
        let long = option
            .get_long()
            .map(|long| format!("--{}", long))
            .unwrap_or_default();

        if !global && (long == "--help" || long == "--version") {
            continue;
        }

        let short = option
            .get_short()
            .map(|short| format!(" -{}", short))
            .unwrap_or_default();

        let value = if option.is_takes_value_set() {
            format!(" <{}>", option.get_name())
        } else {
            String::new()
        };

        let help = option.get_help().unwrap_or_default();
        let help = [
            help,
            "\n",
            option
                .get_long_help()
                .unwrap_or_default()
                .strip_prefix(help)
                .unwrap_or_default(),
        ]
        .concat();

        let mut help = help
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    None
                } else if line.ends_with('.') {
                    Some(line.to_string())
                } else {
                    Some([line, "."].concat())
                }
            })
            .join(" ");

        if let Some(possibles) = option.get_possible_values() {
            let values = possibles
                .iter()
                .map(|value| format!("`{}`", value.get_name()))
                .join(", ");
            help = format!(
                "{}. One of: {}",
                help.strip_suffix('.').unwrap_or(&help),
                values
            );
        }
        let defaults = option
            .get_default_values()
            .iter()
            .map(|value| value.to_string_lossy().to_string())
            .join(", ");
        if !defaults.is_empty() {
            help = format!(
                "{}. Default: {}",
                help.strip_suffix('.').unwrap_or(&help),
                defaults
            );
        }

        count += 1;
        md += &format!("| `{long}{short}{value}` | {help} |\n",);
    }

    if count == 0 {
        String::new()
    } else {
        md
    }
}
