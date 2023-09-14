use cli::Cli;

/// Generates documentation for the CLI in the sibling `stencila` crate
fn main() {
    clap_markdown::print_help_markdown::<Cli>();
}
