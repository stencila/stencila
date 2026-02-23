use eyre::Result;
use stencila_server::preview::PreviewEvent;
use tokio::{sync::mpsc, task::JoinHandle};

/// Default port for the TUI site preview server.
const PREVIEW_PORT: u16 = 9000;

/// Handle to the background site preview task.
///
/// Aborting the task causes the [`stencila_server::preview::PreviewHandle`]
/// inside it to drop, which shuts down the server and cleans the temp dir.
pub struct SitePreviewHandle {
    pub event_rx: mpsc::UnboundedReceiver<PreviewEvent>,
    task: JoinHandle<()>,
}

impl Drop for SitePreviewHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Attempt to spawn a site preview in the background.
///
/// Returns `None` if the workspace has no site configuration (this is not
/// an error — it just means preview is not applicable).
pub fn spawn_preview() -> Result<Option<SitePreviewHandle>> {
    let cfg = stencila_config::get()?;
    if cfg.site.is_none() {
        return Ok(None);
    }

    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let task = tokio::spawn(async move {
        match stencila_server::preview::start_preview(PREVIEW_PORT).await {
            Ok(mut handle) => {
                // `start_preview` returns url/token as struct fields rather
                // than emitting a `ServerReady` event on the channel, so we
                // send it explicitly here for the TUI to display.
                let _ = event_tx.send(PreviewEvent::ServerReady {
                    url: handle.url.clone(),
                    token: handle.token.clone(),
                });
                while let Some(event) = handle.event_rx.recv().await {
                    if event_tx.send(event).is_err() {
                        break;
                    }
                }
            }
            Err(error) => {
                let _ = event_tx.send(PreviewEvent::Error(error.to_string()));
            }
        }
    });

    Ok(Some(SitePreviewHandle { event_rx, task }))
}
