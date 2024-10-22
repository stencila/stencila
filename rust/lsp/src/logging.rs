use std::sync::Mutex;

use async_lsp::{
    lsp_types::{MessageType, ShowMessageParams},
    ClientSocket, LanguageClient,
};
use tracing_subscriber::{
    field::Visit, filter::LevelFilter, fmt, layer::SubscriberExt, registry,
    util::SubscriberInitExt, EnvFilter, Layer,
};

use common::tracing::{field::Field, Event, Level, Subscriber};

/// Setup logging
///
/// This is very similar to, and based on the `cli::logging` module but adds
/// a layer to send info, warn, and error events to the client and has fewer options
/// (e.g. no choice of output format);
pub fn setup(level: LevelFilter, filter: &str, client: ClientSocket) {
    let filter = format!(
        "{}{}{}",
        level,
        if filter.is_empty() { "" } else { "," },
        filter
    );
    let filter_layer = EnvFilter::builder().parse(filter).unwrap_or_default();

    let format_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .pretty();

    let client_layer = ClientLayer::new(client);

    registry()
        .with(filter_layer)
        .with(format_layer)
        .with(client_layer)
        .init();
}

/// A [`tracing_subscriber::Layer`] that forwards events to the LSP client as messages
pub struct ClientLayer {
    client: Mutex<ClientSocket>,
}

impl ClientLayer {
    pub fn new(client: ClientSocket) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

impl<S> Layer<S> for ClientLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(mut client) = self.client.lock() {
            let typ = match *event.metadata().level() {
                Level::INFO => MessageType::INFO,
                Level::WARN => MessageType::WARNING,
                Level::ERROR => MessageType::ERROR,
                _ => return,
            };

            let mut visitor = EventVisitor::default();
            event.record(&mut visitor);
            let mut message = visitor.message;
            if message.is_empty() {
                message = format!("{:?}", event);
            }

            client.show_message(ShowMessageParams { typ, message }).ok();
        }
    }
}

/// Custom [`Event`] visitor to extract the [`Level`] and message of an event
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
            self.message = format!("{:?}", value);
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}
