use crate::binaries;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    Page,
};
use eyre::{Result};
use futures::StreamExt;
use tokio::sync::{Mutex, OnceCell};

static BROWSER: OnceCell<Mutex<Browser>> = OnceCell::const_new();

pub async fn browser() -> Result<&'static Mutex<Browser>> {
    BROWSER
        .get_or_try_init(|| async {
            let chrome = binaries::require("chrome", "*").await?;

            let config = BrowserConfig::builder()
                .chrome_executable(chrome.path)
                .build()
                .expect("Should build config");

            let (browser, mut handler) = Browser::launch(config).await?;
            tokio::task::spawn(async move {
                loop {
                    let _ = handler.next().await.unwrap();
                }
            });

            Ok(Mutex::new(browser))
        })
        .await
}

pub async fn page() -> Result<Page> {
    let page = browser()
        .await?
        .lock()
        .await
        .new_page("about:blank")
        .await?;
    Ok(page)
}
