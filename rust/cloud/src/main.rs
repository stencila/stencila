//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//! Run (in this crate's directory)  with `--all-features` (if necessary) e.g.
//!
//! cargo run --all-features -- --help

use cloud::cli::Command;
cli_utils::mini_main!(Command);
