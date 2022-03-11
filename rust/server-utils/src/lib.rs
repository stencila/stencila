use std::{env, net::SocketAddr};

use axum::{Router, Server};
use events::{subscribe, Subscriber};
use eyre::Result;
use tokio::sync::mpsc;

// Re-exports for consumers of this crate
pub use ::axum;
pub use ::portpicker;
pub use ::serde_json;

/// Get a hostname to use in an externally accessible URL
///
/// This is used when we need to provide an external service with a URL to
/// connect to a server for this instance e.g. Webhooks.
///
/// If the environment variable `STENCILA_HOSTNAME` is defined then that is used,
/// falling back to the public IP address, falling back to `localhost`.
pub async fn hostname() -> String {
    if let Ok(hostname) = env::var("STENCILA_HOSTNAME") {
        hostname
    } else if let Some(ip) = public_ip::addr().await {
        if ip.is_ipv6() {
            // IP6 addresses need to be surrounded in square brackets to use in a URL
            ["[", &ip.to_string(), "]"].concat()
        } else {
            ip.to_string()
        }
    } else {
        "localhost".into()
    }
}

/// Serve with graceful shutdown on Ctrl+C
pub async fn serve_gracefully(ip: [u8; 4], port: u16, router: Router) -> Result<()> {
    let addr = SocketAddr::from((ip, port));
    let server = Server::bind(&addr).serve(router.into_make_service());
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await?;

    Ok(())
}

/// Wait until the interrupt event is received
///
/// Previously this used `tokio::signal::ctrl_c`, but to support having multiple servers
/// (and other tasks) that can be gracefully shutdown uses the "interrupt" event topic instead
/// (because for signals, the last registered handler wins)
async fn shutdown_signal() {
    let (interrupt_sender, mut interrupt_receiver) = mpsc::unbounded_channel();
    subscribe("interrupt", Subscriber::UnboundedSender(interrupt_sender))
        .expect("Unable to subscribe to interrupt event");
    interrupt_receiver.recv().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hostname() {
        // IP address
        let hn = hostname().await;
        println!("{}", hn);
        assert!(hn.contains('.') || hn.contains('['));

        // Domain name
        env::set_var("STENCILA_HOSTNAME", "test.example.org");
        let hn = hostname().await;
        println!("{}", hn);
        assert_eq!(hn, "test.example.org");
    }
}
