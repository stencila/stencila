//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//! Run using `cargo run --all-features` in this crate (`--all-features` is needed to include optional dependencies)
//! The `cfg` flags are just to prevent clippy complaining when running `cargo clippy` without
//! the `--features=cli` flag.

#[cfg(feature = "cli")]
use binaries::commands::Command;

#[cfg(feature = "cli")]
cli_utils::mini_main!(Command);

#[cfg(not(feature = "cli"))]
fn main() {}
