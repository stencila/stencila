//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//! Run (in this crate's directory)  with `--all-features` so that all providers are included e.g.
//!
//! cargo run --all-features -- --help

use server_next::cli::Command;
cli_utils::mini_main!(Command);
