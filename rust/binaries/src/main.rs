//! Mini CLI for testing this crate at the command line without compiling the whole `stencila` binary.
//! Run using `cargo run --all-features` in this crate (`--all-features` is needed to include optional dependencies)

use binaries::commands::Command;
cli_utils::mini_main!(Command);
