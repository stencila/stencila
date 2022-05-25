//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//! Run (in this crate's directory)  with `--all-features` so that all buildpacks are included e.g.
//!
//! cargo run --all-features -- --help

#[cfg(feature = "cli")]
use buildpacks::cli::Command;
#[cfg(feature = "cli")]
cli_utils::mini_main!(Command);
