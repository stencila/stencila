use std::{path::PathBuf, process::exit};

use clap::{Args, Parser, Subcommand};
use eyre::Result;

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Tabulated},
};
use stencila_codecs::{DecodeOptions, EncodeOptions, Format};
use stencila_schema::{Node, NodeType};

/// Manage agent skills
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all skills in the current workspace</dim>
  <b>stencila skills</>

  <dim># Show details about a specific skill</dim>
  <b>stencila skills show</> <g>data-analysis</>

  <dim># Validate a skill by name, directory, or file path</dim>
  <b>stencila skills validate</> <g>data-analysis</>
  <b>stencila skills validate</> <g>.stencila/skills/data-analysis</>

  <dim># Create a new skill</dim>
  <b>stencila skills create</> <g>my-new-skill</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Validate(Validate),
    Create(Create),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Show(show) => show.run().await?,
            Command::Validate(validate) => validate.run().await?,
            Command::Create(create) => create.run().await?,
        }

        Ok(())
    }
}

/// List available skills
///
/// Shows all skills found in the current workspace's `.stencila/skills/` directory.
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all skills in table format</dim>
  <b>stencila skills list</>

  <dim># Output skills as JSON</dim>
  <b>stencila skills list</> <c>--as</> <g>json</>
"
);

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list_current().await;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "License"]);

        for skill in list {
            let license = skill
                .options
                .licenses
                .as_ref()
                .and_then(|licenses| {
                    licenses.first().map(|l| match l {
                        stencila_schema::CreativeWorkVariantOrString::String(s) => s.clone(),
                        _ => String::from("(complex)"),
                    })
                })
                .unwrap_or_default();

            table.add_row([
                Cell::new(&skill.name).add_attribute(Attribute::Bold),
                Cell::new(&skill.description),
                Cell::new(license),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Create a new skill
///
/// Creates a new skill directory with a template SKILL.md in the current
/// workspace's `.stencila/skills/` directory.
#[derive(Debug, Args)]
#[command(after_long_help = CREATE_AFTER_LONG_HELP)]
struct Create {
    /// The name for the new skill
    name: String,
}

pub static CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new skill</dim>
  <b>stencila skills create</> <g>my-new-skill</>
"
);

impl Create {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        // Validate the name first
        let name_errors = super::validate_name(&self.name);
        if !name_errors.is_empty() {
            eprintln!("⚠️  Invalid skill name `{}`:", self.name);
            for error in &name_errors {
                eprintln!("  - {error}");
            }
            exit(1)
        }

        let cwd = std::env::current_dir()?;
        let skills_dir = super::closest_skills_dir(&cwd, true).await?;
        let skill_dir = skills_dir.join(&self.name);

        if skill_dir.exists() {
            eyre::bail!(
                "Skill `{}` already exists at `{}`",
                self.name,
                skill_dir.display()
            );
        }

        tokio::fs::create_dir_all(&skill_dir).await?;

        let skill_md = skill_dir.join("SKILL.md");
        let template = format!(
            "---\nname: {name}\ndescription: TODO\n---\n\nTODO: Add instructions for this skill.\n",
            name = self.name
        );
        tokio::fs::write(&skill_md, template).await?;

        eprintln!(
            "✨ Created skill `{}` at `{}`",
            self.name,
            skill_dir.display()
        );

        Ok(())
    }
}

/// Show a skill
///
/// Displays the full content and metadata of a specific skill.
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the skill to show
    name: String,

    /// The format to show the skill in
    #[arg(long, short, default_value = "md")]
    r#as: Format,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show a skill as Markdown</dim>
  <b>stencila skills show</> <g>data-analysis</>

  <dim># Show a skill as JSON</dim>
  <b>stencila skills show</> <g>data-analysis</> <c>--as</> <g>json</>
"
);

impl Show {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let skills_dir = super::closest_skills_dir(&cwd, false).await?;
        let skill = super::get(&skills_dir, &self.name).await?;

        let content = stencila_codecs::to_string(
            &Node::Skill(skill.inner),
            Some(EncodeOptions {
                format: Some(self.r#as.clone()),
                ..Default::default()
            }),
        )
        .await?;

        Code::new(self.r#as, &content).to_stdout();

        Ok(())
    }
}

/// Validate a skill
///
/// Checks that a skill conforms to the Agent Skills Specification naming
/// and constraint rules. Accepts a skill name, a directory path, or a
/// path to a SKILL.md file.
#[derive(Debug, Args)]
#[command(after_long_help = VALIDATE_AFTER_LONG_HELP)]
struct Validate {
    /// Skill name, directory path, or SKILL.md path
    target: String,
}

pub static VALIDATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Validate a skill by name</dim>
  <b>stencila skills validate</> <g>data-analysis</>

  <dim># Validate a skill directory</dim>
  <b>stencila skills validate</> <g>.stencila/skills/data-analysis</>

  <dim># Validate a SKILL.md file directly</dim>
  <b>stencila skills validate</> <g>.stencila/skills/data-analysis/SKILL.md</>
"
);

impl Validate {
    /// Resolve the target to a SKILL.md path and optional directory name
    async fn resolve_target(&self) -> Result<(PathBuf, Option<String>)> {
        let path = PathBuf::from(&self.target);

        // If target is a path to a SKILL.md file
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("SKILL.md"))
        {
            let dir_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(String::from);
            return Ok((path, dir_name));
        }

        // If target is a directory (containing SKILL.md)
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).map(String::from);
                return Ok((skill_md, dir_name));
            }
            eyre::bail!("No SKILL.md found in directory `{}`", path.display());
        }

        // Otherwise, treat as a skill name — look up in closest skills dir
        let cwd = std::env::current_dir()?;
        let skills_dir = super::closest_skills_dir(&cwd, false).await?;
        let skill = super::get(&skills_dir, &self.target).await?;
        let skill_path = skill.path().to_path_buf();
        let dir_name = skill_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from);
        Ok((skill_path, dir_name))
    }

    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let (skill_md, dir_name) = self.resolve_target().await?;

        let content = tokio::fs::read_to_string(&skill_md).await?;
        let node = stencila_codecs::from_str(
            &content,
            Some(DecodeOptions {
                format: Some(Format::Markdown),
                node_type: Some(NodeType::Skill),
                ..Default::default()
            }),
        )
        .await?;

        let Node::Skill(skill) = node else {
            eyre::bail!("Failed to parse `{}` as a Skill", skill_md.display());
        };

        let errors = super::validate_skill(&skill, dir_name.as_deref());

        if errors.is_empty() {
            eprintln!("🎉 Skill `{}` is valid", skill.name);
            Ok(())
        } else {
            eprintln!(
                "⚠️  Skill `{}` has {} error{}:",
                skill.name,
                errors.len(),
                if errors.len() > 1 { "s" } else { "" }
            );
            for error in &errors {
                eprintln!("  - {error}");
            }
            exit(1)
        }
    }
}
