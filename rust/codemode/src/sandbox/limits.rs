use std::time::Instant;

use rquickjs::AsyncRuntime;

use crate::types::Limits;

/// Configure the memory limit on the QuickJS runtime.
///
/// This is called at sandbox construction since the memory limit applies
/// to the entire runtime lifetime (including global setup).
pub(super) async fn configure_memory_limit(runtime: &AsyncRuntime, limits: Option<&Limits>) {
    let Some(limits) = limits else { return };

    if let Some(max_memory) = limits.max_memory_bytes {
        runtime.set_memory_limit(max_memory as usize).await;
    }
}

/// Install the timeout interrupt handler on the QuickJS runtime.
///
/// Called at the start of `execute()` so the deadline is measured from
/// when code actually runs, not from sandbox construction.
pub(super) async fn install_timeout_handler(runtime: &AsyncRuntime, timeout_ms: Option<u64>) {
    let Some(timeout_ms) = timeout_ms else { return };

    let deadline = Instant::now() + std::time::Duration::from_millis(timeout_ms);
    runtime
        .set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)))
        .await;
}
