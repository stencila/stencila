use cli::Cli;

/// Generates documentation for the CLI in the sibling `stencila` crate
fn main() {
    let help = clap_markdown::help_markdown::<Cli>();

    let mut output = String::new();
    let mut ignore = false;
    for (index, line) in help.lines().enumerate() {
        if index == 0 {
            // Skip the first title line
            continue;
        }

        if ignore && line.starts_with("###### **Arguments") {
            ignore = false;
        }

        if !ignore {
            if line.starts_with("## ") {
                // Promote H2 -> H1
                output.push_str(&line[1..]);
            } else if line.starts_with("###### ") {
                // Remove H6's and just use bolded
                output.push_str(&line[7..]);
            } else {
                output.push_str(&line);
            }
            output.push('\n');
        }

        if line.starts_with("**Usage:** `stencila publish zenodo") {
            ignore = true
        }
    }

    println!(
        r#"---
title: CLI Help
description: Help for the `stencila` CLI 
config:
  publish:
    ghost:
      slug: cli-help
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---
{output}
"#
    )
}
