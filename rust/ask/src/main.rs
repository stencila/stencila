use common::{eyre::Result, tokio};

use ask::{
    Answer, AskLevel, AskOptions, ask, ask_with, ask_with_default, ask_with_default_and_cancel,
};

/// As little CLI for manually testing this crate
/// Run it using `cargo run -p ask`
#[tokio::main]
async fn main() -> Result<()> {
    let got = ask("No default").await?;
    println!("=> {got}");

    let got = ask_with_default("Default is yes", Answer::Yes).await?;
    println!("=> {got}");

    let got = ask_with_default("Default is no", Answer::No).await?;
    println!("=> {got}");

    let got = ask_with_default("Default is cancel", Answer::Cancel).await?;
    println!("=> {got}");

    let got = ask_with_default_and_cancel("Default is yes, cancel is enabled", Answer::Yes).await?;
    println!("=> {got}");

    let got = ask_with_default_and_cancel("Default is no, cancel is enabled", Answer::No).await?;
    println!("=> {got}");

    let got =
        ask_with_default_and_cancel("Default is cancel, cancel is enabled", Answer::Cancel).await?;
    println!("=> {got}");

    let got = ask_with(
        "Warning, proceed?",
        AskOptions {
            level: AskLevel::Warning,
            ..Default::default()
        },
    )
    .await?;
    println!("=> {got}");

    let got = ask_with(
        "Error, bail?",
        AskOptions {
            level: AskLevel::Error,
            ..Default::default()
        },
    )
    .await?;
    println!("=> {got}");

    Ok(())
}
