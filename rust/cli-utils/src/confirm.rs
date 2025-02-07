use std::io::{self, Write};

use common::eyre::Result;

/// Ask the use to confirm with a "y"
#[allow(clippy::print_stdout)]
pub fn confirm(question: &str) -> Result<bool> {
    print!("{question} (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let yes = input.trim().eq_ignore_ascii_case("y");

    Ok(yes)
}
