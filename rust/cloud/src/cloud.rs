use std::{env, fs::read_to_string, io::Write, path::PathBuf};

use common::{
    dirs,
    eyre::{bail, Result},
    serde_json,
    tokio::time::{sleep, Duration},
    tracing,
};
use fs_utils::{open_file_600, remove_if_exists};
use http_utils::CLIENT;

use crate::errors::*;
use crate::types::*;

/// The base URL for Stencila Cloud
const BASE_URL: &str = "https://stencila.fly.dev/api/v1";

/// Get the currently authenticated user, if any
pub fn me() -> Result<Option<User>> {
    let path = user_path();
    if path.exists() {
        let json = read_to_string(path)?;
        let user: User = serde_json::from_str(&json)?;
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

/// Login and return the authenticated user
///
/// Creates a voucher, opens the browser at the `/api/v1/vouchers?create=XXX` page,
/// and then starts polling `/api/v1/vouchers?redeem=XXX` to get the generated
/// API tokens.
pub async fn login(base_url: Option<&str>) -> Result<User> {
    if let Some(user) = me()? {
        tracing::info!("Already logged in as @{}; use `stencila logout` first if you want to login as a different user", user.short_name);
        return Ok(user);
    }

    let base_url = base_url.unwrap_or(BASE_URL);

    let voucher = key_utils::generate("svk");

    let create_url = format!("{}/vouchers?create={}", base_url, voucher);
    tracing::info!("Opening login URL in browser: {}", create_url);
    webbrowser::open(&create_url)?;

    tracing::info!("Waiting for you to login in via browser");
    let redeem_url = format!("{}/vouchers?redeem={}", base_url, voucher);
    loop {
        sleep(Duration::from_millis(1000)).await;

        let response = CLIENT.get(&redeem_url).send().await?;
        if response.status() == 200 {
            let token: ApiToken = response.json().await?;
            let json = serde_json::to_string_pretty(&token)?;
            let mut file = open_file_600(token_path())?;
            file.write_all(json.as_bytes())?;

            let response = CLIENT
                .get(format!("{}/me", base_url))
                .bearer_auth(token.token)
                .send()
                .await?;
            let user: User = response.json().await?;
            let json = serde_json::to_string_pretty(&user)?;
            let mut file = open_file_600(user_path())?;
            file.write_all(json.as_bytes())?;

            tracing::info!(
                "Welcome @{}, you successfully logged in to your Stencila account",
                user.short_name
            );

            return Ok(user);
        } else if response.status() == 202 {
            // Voucher exists but not ready to be redeemed
            eprint!(".");
        } else {
            bail!(
                "While redeeming voucher: {}",
                Error::response_to_string(response).await
            );
        }
    }
}

/// Logout the currently logged in user, if any
///
/// Deletes `token.json`, `user.json`, etc.
pub async fn logout() -> Result<()> {
    match me()? {
        Some(user) => {
            remove_if_exists(token_path())?;
            remove_if_exists(user_path())?;

            tracing::info!(
                "Goodbye @{}, you successfully logged out of your Stencila account",
                user.short_name
            );
        }
        None => {
            tracing::info!("No user currently logged in");
        }
    }
    Ok(())
}

/// Get the path used to store `token.json`, `user.json`, and other files
/// associated with this crate
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap())
        .join("stencila")
}

/// Get the path of `token.json`
fn token_path() -> PathBuf {
    config_dir().join("token.json")
}

/// Get the path of `user.json`
fn user_path() -> PathBuf {
    config_dir().join("user.json")
}

/// Read the current Stencila access token
fn token_read() -> Result<String> {
    let path = token_path();
    if path.exists() {
        let json = read_to_string(token_path())?;
        let token: ApiToken = serde_json::from_str(&json)?;
        Ok(token.token)
    } else {
        bail!("You are not logged in; try doing `stencila login` first");
    }
}
