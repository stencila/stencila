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
mod open;
mod options;
mod pull;
mod push;
mod render;
mod site;
mod status;
mod sync;
mod uninstall;
mod unwatch;
pub mod upgrade;
mod watch;

#[allow(clippy::print_stderr)]
#[cfg(test)]
mod tests {
    use super::*;

    use clap::Parser;
    use stencila_cli_utils::strip_ansi_escapes;

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

    #[rustfmt::skip]
    fn collect_all_after_long_help() -> Vec<(&'static str, &'static str)> {
        vec![
            // CLI module help strings
            ("cli", crate::cli::CLI_AFTER_LONG_HELP),
            ("new", crate::new::CLI_AFTER_LONG_HELP),
            ("convert", crate::convert::CLI_AFTER_LONG_HELP),
            ("merge", crate::merge::CLI_AFTER_LONG_HELP),
            ("sync", crate::sync::CLI_AFTER_LONG_HELP),
            ("push", crate::push::CLI_AFTER_LONG_HELP),
            ("watch", crate::watch::CLI_AFTER_LONG_HELP),
            ("unwatch", crate::unwatch::CLI_AFTER_LONG_HELP),
            ("compile", crate::compile::CLI_AFTER_LONG_HELP),
            ("lint", crate::lint::CLI_AFTER_LONG_HELP),
            ("execute", crate::execute::CLI_AFTER_LONG_HELP),
            ("render", crate::render::CLI_AFTER_LONG_HELP),
            ("preview", crate::open::CLI_AFTER_LONG_HELP),
            ("demo", crate::demo::DEMO_AFTER_LONG_HELP),
            ("upgrade", crate::upgrade::CLI_AFTER_LONG_HELP),
            ("uninstall", crate::uninstall::CLI_AFTER_LONG_HELP),
            ("config", stencila_config::cli::CLI_AFTER_LONG_HELP),
            // Document module help strings
            ("document::init", stencila_document::cli::INIT_AFTER_LONG_HELP),
            ("status", crate::status::CLI_AFTER_LONG_HELP),
            ("document::move", stencila_document::cli::MOVE_AFTER_LONG_HELP),
            ("document::track", stencila_document::cli::TRACK_AFTER_LONG_HELP),
            ("document::untrack", stencila_document::cli::UNTRACK_AFTER_LONG_HELP),
            ("document::clean", stencila_document::cli::CLEAN_AFTER_LONG_HELP),
            ("document::query", stencila_document::cli::QUERY_AFTER_LONG_HELP),
            // DB module help strings
            ("db::new", stencila_node_db::cli::NEW_AFTER_LONG_HELP),
            ("db::add", crate::db::ADD_AFTER_LONG_HELP),
            ("db::remove", crate::db::REMOVE_AFTER_LONG_HELP),
            ("db::query", crate::db::QUERY_AFTER_LONG_HELP),
            ("db::migrate", stencila_node_db::cli::MIGRATE_AFTER_LONG_HELP),
            ("db::migrations", stencila_node_db::cli::MIGRATIONS_AFTER_LONG_HELP),
            // Prompts module help strings
            ("prompts::cli", stencila_prompts::cli::CLI_AFTER_LONG_HELP),
            ("prompts::list", stencila_prompts::cli::LIST_AFTER_LONG_HELP),
            ("prompts::show", stencila_prompts::cli::SHOW_AFTER_LONG_HELP),
            ("prompts::infer", stencila_prompts::cli::INFER_AFTER_LONG_HELP),
            ("prompts::update", stencila_prompts::cli::UPDATE_AFTER_LONG_HELP),
            ("prompts::reset", stencila_prompts::cli::RESET_AFTER_LONG_HELP),
            // Models module help strings
            ("models::cli", stencila_models::cli::CLI_AFTER_LONG_HELP),
            ("models::list", stencila_models::cli::LIST_AFTER_LONG_HELP),
            ("models::run", stencila_models::cli::RUN_AFTER_LONG_HELP),
            // Kernels module help strings
            ("kernels::cli", stencila_kernels::cli::CLI_AFTER_LONG_HELP),
            ("kernels::list", stencila_kernels::cli::LIST_AFTER_LONG_HELP),
            ("kernels::info", stencila_kernels::cli::INFO_AFTER_LONG_HELP),
            ("kernels::packages", stencila_kernels::cli::PACKAGES_AFTER_LONG_HELP),
            ("kernels::execute", stencila_kernels::cli::EXECUTE_AFTER_LONG_HELP),
            ("kernels::evaluate", stencila_kernels::cli::EVALUATE_AFTER_LONG_HELP),
            // Linters module help strings
            ("linters::list", stencila_linters::cli::LIST_AFTER_LONG_HELP),
            ("linters::lint", stencila_linters::cli::LINT_AFTER_LONG_HELP),
            // Codecs module help strings
            ("formats::cli", stencila_codecs::cli::CLI_AFTER_LONG_HELP),
            ("formats::list", stencila_codecs::cli::LIST_AFTER_LONG_HELP),
            // Secrets module help strings
            ("secrets::cli", stencila_secrets::cli::CLI_AFTER_LONG_HELP),
            ("secrets::set", stencila_secrets::cli::SET_AFTER_LONG_HELP),
            ("secrets::delete", stencila_secrets::cli::DELETE_AFTER_LONG_HELP),
            // Tools module help strings
            ("tools::cli", stencila_tools::cli::CLI_AFTER_LONG_HELP),
            ("tools::list", stencila_tools::cli::LIST_AFTER_LONG_HELP),
            ("tools::show", stencila_tools::cli::SHOW_AFTER_LONG_HELP),
            ("tools::install", stencila_tools::cli::INSTALL_AFTER_LONG_HELP),
            ("tools::env", stencila_tools::cli::ENV_AFTER_LONG_HELP),
            ("tools::run", stencila_tools::cli::RUN_AFTER_LONG_HELP),
            // Cloud module help strings
            ("cloud::cli", crate::cloud::CLI_AFTER_LONG_HELP),
            ("cloud::signin", crate::cloud::SIGNIN_AFTER_LONG_HELP),
            ("cloud::signout", crate::cloud::SIGNOUT_AFTER_LONG_HELP),
            ("cloud::status", crate::cloud::STATUS_AFTER_LONG_HELP),
            ("cloud::logs", crate::cloud::LOGS_AFTER_LONG_HELP),
            // Site module help strings
            ("site::cli", crate::site::AFTER_LONG_HELP),
            ("site::show", crate::site::SHOW_AFTER_LONG_HELP),
            ("site::create", crate::site::CREATE_AFTER_LONG_HELP),
            ("site::delete", crate::site::DELETE_AFTER_LONG_HELP),
            ("site::access", crate::site::ACCESS_AFTER_LONG_HELP),
            ("site::access::public", crate::site::ACCESS_PUBLIC_AFTER_LONG_HELP),
            ("site::access::password", crate::site::ACCESS_PASSWORD_AFTER_LONG_HELP),
            ("site::access::team", crate::site::ACCESS_TEAM_AFTER_LONG_HELP),
            ("site::password", crate::site::PASSWORD_AFTER_LONG_HELP),
            ("site::password::set", crate::site::PASSWORD_SET_AFTER_LONG_HELP),
            ("site::password::clear", crate::site::PASSWORD_CLEAR_AFTER_LONG_HELP),
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
