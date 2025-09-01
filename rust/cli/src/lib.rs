#![recursion_limit = "256"]

mod cli;
pub use crate::cli::{Cli, Command};

mod cloud;
mod compile;
mod convert;
mod db;
mod demo;
pub mod errors;
mod execute;
mod lint;
pub mod logging;
mod merge;
mod new;
mod options;
mod preview;
mod render;
mod sync;
mod uninstall;
pub mod upgrade;

#[allow(clippy::print_stderr)]
#[cfg(test)]
mod tests {
    use super::*;

    use clap::Parser;
    use cli_utils::strip_ansi_escapes;

    /// Test that all CLI example commands in AFTER_LONG_HELP strings parse correctly
    ///
    /// This test:
    /// 1. Collects all AFTER_LONG_HELP constants from all CLI modules
    /// 2. Extracts example commands starting with "stencila"
    /// 3. Strips ANSI escape sequences and parses quoted arguments
    /// 4. Validates each command using Cli::try_parse_from()
    /// 5. Skips commands that fail due to validation constraints (not parsing issues)
    #[test]
    fn test_after_long_help_examples() {
        let help_strings = collect_all_after_long_help();
        let mut successful_commands = 0;

        for (command_name, help_text) in help_strings {
            let examples = extract_stencila_commands(help_text);

            for example in examples {
                let args = parse_command_line(&example);

                // Special handling for help commands
                if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h") {
                    continue;
                }

                match Cli::try_parse_from(&args) {
                    Ok(_) => {
                        successful_commands += 1;
                    }
                    Err(e) => {
                        panic!("Command `{command_name}` example `{example}` failed: {e}");
                    }
                }
            }
        }

        eprintln!("✅ Validated {successful_commands} CLI example commands successfully!");
    }

    fn collect_all_after_long_help() -> Vec<(&'static str, &'static str)> {
        vec![
            // CLI module help strings
            ("cli", crate::cli::CLI_AFTER_LONG_HELP),
            ("new", crate::new::CLI_AFTER_LONG_HELP),
            ("convert", crate::convert::CLI_AFTER_LONG_HELP),
            ("merge", crate::merge::CLI_AFTER_LONG_HELP),
            ("sync", crate::sync::CLI_AFTER_LONG_HELP),
            ("compile", crate::compile::CLI_AFTER_LONG_HELP),
            ("lint", crate::lint::CLI_AFTER_LONG_HELP),
            ("execute", crate::execute::CLI_AFTER_LONG_HELP),
            ("render", crate::render::CLI_AFTER_LONG_HELP),
            ("preview", crate::preview::CLI_AFTER_LONG_HELP),
            ("demo", crate::demo::DEMO_AFTER_LONG_HELP),
            ("upgrade", crate::upgrade::CLI_AFTER_LONG_HELP),
            ("uninstall", crate::uninstall::CLI_AFTER_LONG_HELP),
            // Document module help strings
            ("document::init", document::cli::INIT_AFTER_LONG_HELP),
            ("document::config", document::cli::CONFIG_AFTER_LONG_HELP),
            ("document::status", document::cli::STATUS_AFTER_LONG_HELP),
            ("document::move", document::cli::MOVE_AFTER_LONG_HELP),
            ("document::track", document::cli::TRACK_AFTER_LONG_HELP),
            ("document::untrack", document::cli::UNTRACK_AFTER_LONG_HELP),
            ("document::clean", document::cli::CLEAN_AFTER_LONG_HELP),
            ("document::query", document::cli::QUERY_AFTER_LONG_HELP),
            // DB module help strings
            ("db::new", node_db::cli::NEW_AFTER_LONG_HELP),
            ("db::add", crate::db::ADD_AFTER_LONG_HELP),
            ("db::remove", crate::db::REMOVE_AFTER_LONG_HELP),
            ("db::query", crate::db::QUERY_AFTER_LONG_HELP),
            ("db::migrate", node_db::cli::MIGRATE_AFTER_LONG_HELP),
            ("db::migrations", node_db::cli::MIGRATIONS_AFTER_LONG_HELP),
            // Prompts module help strings
            ("prompts::cli", prompts::cli::CLI_AFTER_LONG_HELP),
            ("prompts::list", prompts::cli::LIST_AFTER_LONG_HELP),
            ("prompts::show", prompts::cli::SHOW_AFTER_LONG_HELP),
            ("prompts::infer", prompts::cli::INFER_AFTER_LONG_HELP),
            ("prompts::update", prompts::cli::UPDATE_AFTER_LONG_HELP),
            ("prompts::reset", prompts::cli::RESET_AFTER_LONG_HELP),
            // Models module help strings
            ("models::cli", models::cli::CLI_AFTER_LONG_HELP),
            ("models::list", models::cli::LIST_AFTER_LONG_HELP),
            ("models::run", models::cli::RUN_AFTER_LONG_HELP),
            // Kernels module help strings
            ("kernels::cli", kernels::cli::CLI_AFTER_LONG_HELP),
            ("kernels::list", kernels::cli::LIST_AFTER_LONG_HELP),
            ("kernels::info", kernels::cli::INFO_AFTER_LONG_HELP),
            ("kernels::packages", kernels::cli::PACKAGES_AFTER_LONG_HELP),
            ("kernels::execute", kernels::cli::EXECUTE_AFTER_LONG_HELP),
            ("kernels::evaluate", kernels::cli::EVALUATE_AFTER_LONG_HELP),
            // Linters module help strings
            ("linters::list", stencila_linters::cli::LIST_AFTER_LONG_HELP),
            ("linters::lint", stencila_linters::cli::LINT_AFTER_LONG_HELP),
            // Codecs module help strings
            ("formats::cli", codecs::cli::CLI_AFTER_LONG_HELP),
            ("formats::list", codecs::cli::LIST_AFTER_LONG_HELP),
            // Plugins module help strings
            ("plugins::cli", plugins::cli::CLI_AFTER_LONG_HELP),
            // Secrets module help strings
            ("secrets::cli", secrets::cli::CLI_AFTER_LONG_HELP),
            ("secrets::set", secrets::cli::SET_AFTER_LONG_HELP),
            ("secrets::delete", secrets::cli::DELETE_AFTER_LONG_HELP),
            // Tools module help strings
            ("tools::cli", tools::cli::CLI_AFTER_LONG_HELP),
            ("tools::list", tools::cli::LIST_AFTER_LONG_HELP),
            ("tools::show", tools::cli::SHOW_AFTER_LONG_HELP),
            ("tools::install", tools::cli::INSTALL_AFTER_LONG_HELP),
            ("tools::env", tools::cli::ENV_AFTER_LONG_HELP),
            ("tools::run", tools::cli::RUN_AFTER_LONG_HELP),
            // Cloud module help strings
            ("cloud::cli", crate::cloud::CLI_AFTER_LONG_HELP),
            ("cloud::signin", crate::cloud::SIGNIN_AFTER_LONG_HELP),
            ("cloud::signout", crate::cloud::SIGNOUT_AFTER_LONG_HELP),
            ("cloud::status", crate::cloud::STATUS_AFTER_LONG_HELP),
        ]
    }

    fn extract_stencila_commands(help_text: &str) -> Vec<String> {
        let mut commands = Vec::new();

        for line in help_text.lines() {
            let trimmed = line.trim();

            // First strip ANSI sequences to see the clean text
            let clean_line = strip_ansi_escapes(trimmed);

            // Look for lines that start with stencila (ignoring leading whitespace and comments)
            if clean_line.trim_start().starts_with("stencila") {
                // Extract the command from the clean line
                let command = clean_line.trim().to_string();
                commands.push(command);
            }
        }

        commands
    }

    fn parse_command_line(command: &str) -> Vec<String> {
        // Handle quoted arguments properly
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;
        let chars = command.chars().peekable();

        for ch in chars {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }
}
