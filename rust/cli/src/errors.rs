use common::{eyre::Result, itertools::Itertools};

/// Setup error reporting
///
/// This function is for runtime configuration of terminal error reporting based on
/// CLI arguments provided by the user.
///
/// The `details` argument is a comma separated list of report sections to show:
///
/// - `location`: The source file location the error occurred
/// - `span`: The span trace
/// - `env`: Instructions on the environment variables to set for a backtrace
///
/// Use `details = "none"` for none of the above and `details = "auto"` to use
/// all of the above when in development.
///
/// The `link` argument configures whether or not to provide an issue reporting link.
pub fn setup(details: &str, link: bool) -> Result<()> {
    let all = vec!["location", "span", "env"];

    let details = match details {
        "auto" => {
            if cfg!(debug_assertions) {
                all
            } else {
                vec![]
            }
        }
        "all" => all,
        "none" => vec![],
        _ => details.split(',').collect_vec(),
    };

    let mut builder = color_eyre::config::HookBuilder::default()
        .display_location_section(details.contains(&"location"))
        .display_env_section(details.contains(&"env"));

    if !details.contains(&"span") && !link {
        std::env::set_var("RUST_SPANTRACE", "0");
    }

    if link {
        builder = builder
            .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
            .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
            .add_issue_metadata("os", std::env::consts::OS);
    }

    builder.install()?;

    Ok(())
}
