use common::{eyre::Result, tokio};

use ask::{
    Answer, AskLevel, AskOptions, ask, ask_for_password, ask_with, ask_with_default,
    ask_with_default_and_cancel, setup_cli,
};

/// As little CLI for manually testing this crate
/// Run it using `cargo run -p ask`
#[allow(clippy::print_stderr)]
#[tokio::main]
async fn main() -> Result<()> {
    // Setup up CLI provider
    setup_cli(None).await?;

    eprintln!("\n=== `ask` examples ===");

    let got = ask("No default").await?;
    eprintln!("=> {got}");

    let got =
        ask("This is a very long prompt that should wrap nicely when displayed on the terminal. To make it even longer, I'll repeat that. This is a very long prompt that should wrap nicely when displayed on the terminal. Do you agree?")
            .await?;
    eprintln!("=> {got}");

    eprintln!("\n=== `ask_with_default` examples ===");

    let got = ask_with_default("Default is yes", Answer::Yes).await?;
    eprintln!("=> {got}");

    let got = ask_with_default("Default is no", Answer::No).await?;
    eprintln!("=> {got}");

    let got = ask_with_default("Default is cancel", Answer::Cancel).await?;
    eprintln!("=> {got}");

    eprintln!("\n=== `ask_with_default_and_cancel` examples ===");

    let got = ask_with_default_and_cancel("Default is yes, cancel is enabled", Answer::Yes).await?;
    eprintln!("=> {got}");

    let got = ask_with_default_and_cancel("Default is no, cancel is enabled", Answer::No).await?;
    eprintln!("=> {got}");

    let got =
        ask_with_default_and_cancel("Default is cancel, cancel is enabled", Answer::Cancel).await?;
    eprintln!("=> {got}");

    eprintln!("\n=== `ask_with` examples ===");

    let got = ask_with(
        "Warning, proceed?",
        AskOptions {
            level: AskLevel::Warning,
            ..Default::default()
        },
    )
    .await?;
    eprintln!("=> {got}");

    let got = ask_with(
        "Error, bail?",
        AskOptions {
            level: AskLevel::Error,
            ..Default::default()
        },
    )
    .await?;
    eprintln!("=> {got}");

    eprintln!("\n=== `password` examples ===");

    let password = ask_for_password("Enter your password").await?;
    eprintln!("=> Password length: {}", password.len());

    let password = ask_for_password("This is a very long prompt that should wrap nicely when displayed on the terminal. Please enter your super secret password").await?;
    eprintln!("=> Password length: {}", password.len());

    Ok(())
}
