use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crossterm::event::{Event, EventStream as CrosstermEventStream};
use futures::StreamExt;
use tokio::sync::mpsc;

/// Events consumed by the main application loop.
#[derive(Debug)]
pub enum AppEvent {
    /// A terminal event from crossterm (key press, resize, etc.).
    Terminal(Event),
    /// A periodic tick for animations and time-based updates.
    Tick,
}

/// Reads crossterm events and periodic ticks, forwarding them through an mpsc channel.
pub struct EventReader {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventReader {
    /// Spawn background tasks that read terminal events and emit ticks.
    ///
    /// If the terminal event stream ends or errors, both tasks shut down
    /// so the main loop's `next()` returns `None` instead of spinning on ticks.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let shutdown = Arc::new(AtomicBool::new(false));

        // Terminal event reader
        let event_tx = tx.clone();
        let reader_shutdown = Arc::clone(&shutdown);
        tokio::spawn(async move {
            let mut stream = CrosstermEventStream::new();
            while let Some(Ok(event)) = stream.next().await {
                if event_tx.send(AppEvent::Terminal(event)).is_err() {
                    break;
                }
            }
            // Signal the tick task to stop
            reader_shutdown.store(true, Ordering::Release);
        });

        // Tick timer — stops when the event reader shuts down
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
            loop {
                interval.tick().await;
                if shutdown.load(Ordering::Acquire) {
                    break;
                }
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
            // Dropping `tx` here closes the channel, causing `next()` to return `None`.
        });

        Self { rx }
    }

    /// Receive the next event, waiting until one is available.
    ///
    /// Returns `None` when all sender tasks have shut down.
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
