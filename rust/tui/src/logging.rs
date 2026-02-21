use tokio::sync::mpsc;
use tracing::{Event, Level, Subscriber, field::Field};
use tracing_subscriber::{
    EnvFilter, Layer, field::Visit, filter::LevelFilter, layer::SubscriberExt, registry,
    util::SubscriberInitExt,
};

/// Set up tracing for the TUI.
///
/// Installs a global subscriber that captures log events and sends them
/// through a channel instead of writing to stderr (which would clobber
/// the terminal UI). Returns the receiving end of the channel.
pub fn setup(level: LevelFilter, filter: &str) -> mpsc::UnboundedReceiver<String> {
    let filter = format!(
        "{}{}{}",
        level,
        if filter.is_empty() { "" } else { "," },
        filter
    );
    let filter_layer = EnvFilter::builder().parse(filter).unwrap_or_default();

    let (tx, rx) = mpsc::unbounded_channel();
    let tui_layer = TuiLayer { sender: tx };

    // Use try_init() instead of init() to avoid panicking if a global
    // subscriber was already installed (e.g. in embedding/integration contexts).
    let _ = registry().with(filter_layer).with(tui_layer).try_init();

    rx
}

/// A [`tracing_subscriber::Layer`] that forwards events to the TUI as system messages.
struct TuiLayer {
    sender: mpsc::UnboundedSender<String>,
}

impl<S> Layer<S> for TuiLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = match *event.metadata().level() {
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
            _ => return,
        };

        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);
        if visitor.message.is_empty() {
            visitor.message = format!("{event:?}");
        }

        let formatted = format!("{level}: {}", visitor.message);
        // Ignore send errors — the receiver may have been dropped during shutdown.
        let _ = self.sender.send(formatted);
    }
}

/// Custom [`Event`] visitor to extract the message field.
#[derive(Default)]
struct EventVisitor {
    message: String,
}

impl Visit for EventVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}
