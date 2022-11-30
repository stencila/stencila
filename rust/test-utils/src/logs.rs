use common::tracing;

/// Print log entries
///
/// Many of the sibling crates use `tracing` and being able to see log entries
/// can be very useful during testing.
///
/// Prints entries to stderr. Use `cargo test -- --nocapture`.
pub fn print_logs_level(level: tracing::Level) {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(level)
        .init()
}

/// Print DEBUG and above log entries
pub fn print_logs() {
    print_logs_level(tracing::Level::DEBUG)
}
