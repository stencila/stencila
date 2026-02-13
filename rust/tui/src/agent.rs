use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use stencila_agents::convenience::create_session;
use stencila_agents::types::EventKind;
use tokio::sync::mpsc;

/// Shared progress state for a running agent exchange, updated by the
/// background event-draining task.
#[derive(Debug, Default)]
struct AgentProgress {
    /// Accumulated assistant text so far.
    text: String,
    /// Whether the exchange has completed (success or failure).
    is_complete: bool,
    /// An error message, if the exchange failed.
    error: Option<String>,
}

/// A running agent exchange, analogous to [`crate::shell::RunningCommand`].
///
/// The TUI polls this on each tick to stream incremental text updates
/// and detect completion.
pub struct RunningAgentExchange {
    progress: Arc<Mutex<AgentProgress>>,
    cancelled: Arc<AtomicBool>,
}

impl RunningAgentExchange {
    /// Return the current accumulated text.
    pub fn current_text(&self) -> String {
        self.progress
            .lock()
            .map(|g| g.text.clone())
            .unwrap_or_default()
    }

    /// If the exchange is complete, return the final text and optional error.
    ///
    /// Returns `None` if still running.
    pub fn try_take_result(&self) -> Option<AgentExchangeResult> {
        let guard = self.progress.lock().ok()?;
        if guard.is_complete {
            Some(AgentExchangeResult {
                text: guard.text.clone(),
                error: guard.error.clone(),
            })
        } else {
            None
        }
    }

    /// Soft-cancel: stop updating the UI with further events, but let the
    /// agent session finish in the background so it remains usable for
    /// future messages.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

/// The final result of a completed agent exchange.
pub struct AgentExchangeResult {
    /// Full assistant text.
    pub text: String,
    /// Error message, if any.
    pub error: Option<String>,
}

/// Commands sent to the background agent task.
enum AgentCommand {
    Submit {
        text: String,
        progress: Arc<Mutex<AgentProgress>>,
        cancelled: Arc<AtomicBool>,
    },
}

/// Handle for submitting messages to the background agent task.
///
/// Owns the sending half of the command channel. Dropping this handle
/// signals the background task to shut down.
pub struct AgentHandle {
    tx: mpsc::UnboundedSender<AgentCommand>,
}

impl AgentHandle {
    /// Spawn the background agent task and return a handle.
    ///
    /// If `model` is `Some((provider, model_id))`, the agent session will
    /// use that specific model instead of the default. The session is created
    /// lazily on the first submit. Returns `None` if no Tokio runtime is
    /// available (e.g. in synchronous tests).
    pub fn spawn(model: Option<(String, String)>) -> Option<Self> {
        // Check that a tokio runtime is available before spawning
        let _handle = tokio::runtime::Handle::try_current().ok()?;
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(agent_task(rx, model));
        Some(Self { tx })
    }

    /// Submit a chat message to the agent. Returns a `RunningAgentExchange`
    /// for polling, or `None` if the background task has shut down.
    pub fn submit(&self, text: String) -> Option<RunningAgentExchange> {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        let cancelled = Arc::new(AtomicBool::new(false));

        let exchange = RunningAgentExchange {
            progress: Arc::clone(&progress),
            cancelled: Arc::clone(&cancelled),
        };

        self.tx
            .send(AgentCommand::Submit {
                text,
                progress,
                cancelled,
            })
            .ok()?;

        Some(exchange)
    }
}

/// Background task that owns the agent session.
///
/// Waits for `Submit` commands, runs `session.submit()`, and drains
/// events into the shared `AgentProgress`. The session is created lazily
/// on the first submit so that startup errors are surfaced to the user.
async fn agent_task(
    mut rx: mpsc::UnboundedReceiver<AgentCommand>,
    model: Option<(String, String)>,
) {
    // Session and event receiver are created lazily on first submit.
    let mut session = None;
    let mut event_rx = None;

    while let Some(AgentCommand::Submit {
        text,
        progress,
        cancelled,
    }) = rx.recv().await
    {
        // Lazy session init
        if session.is_none() {
            let (provider, model_id) = match &model {
                Some((p, m)) => (Some(p.as_str()), Some(m.as_str())),
                None => (None, None),
            };
            match create_session(provider, model_id).await {
                Ok((s, er)) => {
                    session = Some(s);
                    event_rx = Some(er);
                }
                Err(e) => {
                    if let Ok(mut g) = progress.lock() {
                        g.error = Some(e.to_string());
                        g.is_complete = true;
                    }
                    continue;
                }
            }
        }

        let sess = session.as_mut().expect("session initialized above");
        let ev_rx = event_rx.as_mut().expect("event_rx initialized above");

        // Pin the submit future so we can poll it in tokio::select!
        let mut submit_fut = Box::pin(sess.submit(&text));
        let mut submit_done = false;
        let mut submit_result: Option<Result<(), stencila_agents::error::AgentError>> = None;

        loop {
            tokio::select! {
                biased;

                event = ev_rx.recv() => {
                    let Some(event) = event else {
                        // Event channel closed — session dropped
                        if let Ok(mut g) = progress.lock() {
                            g.is_complete = true;
                        }
                        break;
                    };

                    if cancelled.load(Ordering::Acquire) {
                        // Soft cancel: keep draining events so the channel
                        // doesn't back up, but don't update progress.
                        continue;
                    }

                    process_event(&event, &progress);
                }

                result = &mut submit_fut, if !submit_done => {
                    submit_done = true;
                    submit_result = Some(result);
                }
            }

            // Once submit is done, drain any remaining buffered events
            // using non-blocking try_recv to avoid stalling if the channel
            // is already empty.
            if submit_done {
                while let Ok(event) = ev_rx.try_recv() {
                    if cancelled.load(Ordering::Acquire) {
                        continue;
                    }
                    process_event(&event, &progress);
                }

                // Record any submit error
                if let Some(Err(e)) = &submit_result
                    && let Ok(mut g) = progress.lock()
                {
                    g.error = Some(e.to_string());
                }

                // If cancelled, mark as cancelled
                if cancelled.load(Ordering::Acquire) {
                    if let Ok(mut g) = progress.lock() {
                        if g.text.is_empty() {
                            g.text = "[cancelled]".to_string();
                        }
                        g.is_complete = true;
                    }
                } else if let Ok(mut g) = progress.lock() {
                    g.is_complete = true;
                }

                break;
            }
        }
    }
}

/// Process a single session event, updating the shared progress.
fn process_event(
    event: &stencila_agents::types::SessionEvent,
    progress: &Arc<Mutex<AgentProgress>>,
) {
    match event.kind {
        EventKind::AssistantTextDelta => {
            if let Some(serde_json::Value::String(delta)) = event.data.get("delta")
                && let Ok(mut g) = progress.lock()
            {
                g.text.push_str(delta);
            }
        }
        EventKind::AssistantTextEnd => {
            // Use the final text as the canonical output. This handles both
            // non-streaming providers (where no deltas are received) and acts
            // as a reconciliation in case any streamed deltas were incomplete.
            if let Some(serde_json::Value::String(text)) = event.data.get("text")
                && let Ok(mut g) = progress.lock()
            {
                text.clone_into(&mut g.text);
            }
        }
        EventKind::Error => {
            let message = event
                .data
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            if let Ok(mut g) = progress.lock() {
                g.error = Some(message);
            }
        }
        // Tool events, session events, etc. — no UI update needed for v1
        _ => {}
    }
}
