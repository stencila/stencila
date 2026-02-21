use std::time::Duration;

pub use indicatif::{self, ProgressBar, ProgressStyle};

const TICK_INTERVAL: Duration = Duration::from_millis(100);
const BAR_CHARS: &str = "━╸─";

/// Create a spinner with a green spinner icon and the given message.
pub fn new_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("valid template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(TICK_INTERVAL);
    pb
}

/// Create a progress bar sized in bytes (shows `bytes/total_bytes`).
pub fn new_bytes_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .expect("valid template")
            .progress_chars(BAR_CHARS),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(TICK_INTERVAL);
    pb
}

/// Create a progress bar sized in items (shows `pos/len unit`).
pub fn new_items_bar(total: u64, unit: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    let template = format!(
        "{{spinner:.green}} {{elapsed_precise}} {{bar:40.cyan/blue}} {{pos}}/{{len}} {unit} ({{eta}})"
    );
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&template)
            .expect("valid template")
            .progress_chars(BAR_CHARS),
    );
    pb.enable_steady_tick(TICK_INTERVAL);
    pb
}
