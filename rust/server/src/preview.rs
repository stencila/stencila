use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::Result;
use stencila_document::{Document, ExecuteOptions};
use stencila_web_dist::web_base_localhost;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{ServeOptions, SiteMessage, get_server_token, serve};

/// Events emitted by a running site preview.
#[derive(Debug, Clone)]
pub enum PreviewEvent {
    /// Server is ready at the given URL with the given authentication token.
    ServerReady { url: String, token: String },
    /// Files changed and are being re-rendered.
    Rerendering { files: Vec<String> },
    /// Re-render completed successfully.
    RerenderComplete,
    /// A non-fatal error occurred (e.g. a single document failed to render).
    Error(String),
}

/// Handle to a running site preview.
///
/// Holds the resources that keep the preview alive. Dropping (or calling
/// [`PreviewHandle::shutdown`]) stops the server and cleans up the temporary
/// directory.
pub struct PreviewHandle {
    pub event_rx: mpsc::UnboundedReceiver<PreviewEvent>,
    pub url: String,
    pub token: String,
    server_shutdown_tx: Option<oneshot::Sender<()>>,
    watch_shutdown_tx: Option<oneshot::Sender<()>>,
    _temp_dir: tempfile::TempDir,
}

impl PreviewHandle {
    pub fn shutdown(mut self) {
        self.send_shutdown();
    }

    fn send_shutdown(&mut self) {
        if let Some(tx) = self.watch_shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.server_shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for PreviewHandle {
    fn drop(&mut self) {
        self.send_shutdown();
    }
}

/// Render the site to the output directory.
///
/// This is the shared render logic used by both CLI `site preview` and
/// TUI auto-preview. It calls [`stencila_site::render`] with the standard
/// document decode function.
///
/// If `changed_paths` is provided, only re-render documents matching those
/// paths. If `None`, render all documents (used for config changes and
/// initial render).
///
/// An optional `progress` sender can be provided to receive
/// [`stencila_site::RenderProgress`] events (used by the CLI for progress
/// bars).
pub async fn render_site(
    source: &Path,
    output: &Path,
    port: u16,
    changed_paths: Option<&[PathBuf]>,
    progress: Option<mpsc::Sender<stencila_site::RenderProgress>>,
) -> Result<()> {
    let base_url = format!("http://localhost:{port}");
    let web_base = web_base_localhost(port);

    stencila_site::render(
        source,
        output,
        &base_url,
        Some(web_base.as_str()),
        None,          // route_filter
        None,          // path_filter
        changed_paths, // source_files
        progress,
        |doc_path, arguments: HashMap<String, String>| async move {
            let doc = Document::open(&doc_path, None).await?;
            let arguments: Vec<(&str, &str)> = arguments
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect();
            doc.call(&arguments, ExecuteOptions::default()).await?;
            Ok(doc.root().await)
        },
    )
    .await?;

    Ok(())
}

/// Watch for file and config changes, re-rendering as needed.
///
/// Runs until the `shutdown_rx` fires or `Ctrl+C` is received.
/// Sends [`PreviewEvent`]s on `event_tx` for each re-render cycle.
pub async fn watch_and_rerender(
    workspace_root: &Path,
    site_root: &Path,
    output: &Path,
    port: u16,
    site_notify: broadcast::Sender<SiteMessage>,
    event_tx: mpsc::UnboundedSender<PreviewEvent>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let mut config_receiver = stencila_config::watch(workspace_root).await?;
    let mut site_receiver = stencila_site::watch(site_root, Some(output)).await?;

    enum RenderTrigger {
        Config,
        Site { paths: Vec<String> },
    }
    let mut pending_render: Option<(tokio::task::JoinHandle<Result<()>>, RenderTrigger)> = None;

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                if let Some((handle, _)) = pending_render.take() {
                    handle.abort();
                }
                break;
            }

            Some(result) = async {
                match config_receiver.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Ok(_new_config) => {
                        if let Some((handle, _)) = pending_render.take() {
                            handle.abort();
                            let _ = handle.await;
                        }
                        let _ = event_tx.send(PreviewEvent::Rerendering {
                            files: vec!["(config)".to_string()],
                        });
                        let site_root = site_root.to_path_buf();
                        let output = output.to_path_buf();
                        let handle = tokio::spawn(async move {
                            render_site(&site_root, &output, port, None, None).await
                        });
                        pending_render = Some((handle, RenderTrigger::Config));
                    }
                    Err(error) => {
                        let _ = event_tx.send(PreviewEvent::Error(
                            format!("Config error: {error}"),
                        ));
                    }
                }
            }

            Some(event) = site_receiver.recv() => {
                let nav_override_changed = event.paths.iter().any(|path| matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("_nav.yaml" | "_nav.yml" | "_nav.toml" | "_nav.json")
                ));

                let changed_paths = if nav_override_changed {
                    None
                } else {
                    Some(event.paths.clone())
                };
                let file_names: Vec<String> = event.paths.iter()
                    .filter_map(|p| p.file_name())
                    .filter_map(|n| n.to_str())
                    .map(String::from)
                    .collect();

                if let Some((handle, _)) = pending_render.take() {
                    handle.abort();
                    let _ = handle.await;
                }

                let _ = event_tx.send(PreviewEvent::Rerendering { files: file_names });

                let site_root = site_root.to_path_buf();
                let output = output.to_path_buf();
                let handle = tokio::spawn(async move {
                    render_site(
                        &site_root,
                        &output,
                        port,
                        changed_paths.as_deref(),
                        None,
                    )
                    .await
                });
                let notify_paths: Vec<String> = event.paths.iter()
                    .filter_map(|p| p.to_str())
                    .map(String::from)
                    .collect();
                pending_render = Some((handle, RenderTrigger::Site { paths: notify_paths }));
            }

            Some(result) = async {
                match &mut pending_render {
                    Some((handle, _)) => Some(handle.await),
                    None => std::future::pending().await,
                }
            } => {
                let trigger = pending_render.take().map(|(_, t)| t);
                match result {
                    Ok(Ok(())) => {
                        match trigger {
                            Some(RenderTrigger::Config) => {
                                let _ = site_notify.send(SiteMessage::ConfigChange);
                            }
                            Some(RenderTrigger::Site { paths }) => {
                                let _ = site_notify.send(SiteMessage::SiteChange { paths });
                            }
                            None => {}
                        }
                        let _ = event_tx.send(PreviewEvent::RerenderComplete);
                    }
                    Ok(Err(error)) => {
                        let _ = event_tx.send(PreviewEvent::Error(
                            format!("Render error: {error}"),
                        ));
                    }
                    Err(_) => {
                        // Task was aborted (cancelled), expected
                    }
                }
            }

            else => break,
        }
    }

    Ok(())
}

/// Start a site preview: render, serve, and watch for changes.
///
/// Returns a [`PreviewHandle`] that keeps the server running. Drop the
/// handle (or call [`PreviewHandle::shutdown`]) to stop everything.
///
/// Returns an error if the initial render fails.
pub async fn start_preview(port: u16) -> Result<PreviewHandle> {
    let cfg = stencila_config::get()?;

    let site_root = cfg
        .site
        .as_ref()
        .and_then(|s| s.root.as_ref())
        .map(|r| cfg.workspace_dir.join(r))
        .unwrap_or_else(|| cfg.workspace_dir.clone());

    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_path_buf();

    // Initial render
    render_site(&site_root, &temp_path, port, None, None).await?;

    let server_token = get_server_token();

    // Shutdown channels
    let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel();
    let (watch_shutdown_tx, watch_shutdown_rx) = oneshot::channel();

    // Site notification broadcast channel
    let (site_notify_tx, _) = broadcast::channel::<SiteMessage>(16);
    let site_notify_tx_clone = site_notify_tx.clone();

    // Start server
    let serve_dir = temp_path.clone();
    let server_token_clone = server_token.clone();
    tokio::spawn(async move {
        let options = ServeOptions {
            dir: serve_dir.clone(),
            port,
            server_token: Some(server_token_clone),
            no_startup_message: true,
            shutdown_receiver: Some(server_shutdown_rx),
            static_dir: Some(serve_dir),
            site_notify: Some(site_notify_tx_clone),
            ..Default::default()
        };
        let _ = serve(options).await;
    });

    let url = format!("http://localhost:{port}");

    // Event channel for the watch loop
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // Spawn watch loop
    let workspace_root = cfg.workspace_dir.clone();
    let watch_site_root = site_root.clone();
    let watch_output = temp_path.clone();
    tokio::spawn(async move {
        let _ = watch_and_rerender(
            &workspace_root,
            &watch_site_root,
            &watch_output,
            port,
            site_notify_tx,
            event_tx,
            watch_shutdown_rx,
        )
        .await;
    });

    Ok(PreviewHandle {
        event_rx,
        url,
        token: server_token,
        server_shutdown_tx: Some(server_shutdown_tx),
        watch_shutdown_tx: Some(watch_shutdown_tx),
        _temp_dir: temp_dir,
    })
}
