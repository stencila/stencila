//! Local HTTP callback server for OAuth redirect flows.
//!
//! Binds a TCP listener on a specified port, waits for a single GET request
//! containing an authorization code (and optionally state), serves a
//! success page, then shuts down.

use std::io::Write;
use std::net::TcpListener;

use eyre::{Result, eyre};
use url::Url;

/// Errors that can occur while waiting for the OAuth callback.
#[derive(Debug)]
pub enum CallbackError {
    /// The OAuth provider returned an explicit error (e.g. `access_denied`).
    /// The user denied consent or the provider rejected the request.
    OAuthDenied {
        /// The OAuth error code (e.g. `access_denied`).
        error: String,
        /// Optional human-readable description from the provider.
        description: String,
    },
    /// A transport or setup failure (port bind, connection, malformed request).
    Transport(eyre::Report),
}

impl std::fmt::Display for CallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OAuthDenied { error, description } => {
                write!(f, "OAuth error: {error} - {description}")
            }
            Self::Transport(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CallbackError {}

/// Parameters extracted from the OAuth redirect callback.
#[derive(Debug, Clone)]
pub struct CallbackParams {
    /// The authorization code from the provider.
    pub code: String,
    /// The state parameter, if present.
    pub state: Option<String>,
}

/// HTML page served to the browser after a successful callback.
const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Authorization Complete</title></head>
<body style="font-family: sans-serif; text-align: center; padding: 2em;">
<h2>Authorization complete</h2>
<p>You can close this window and return to the terminal.</p>
</body>
</html>"#;

/// Start a local HTTP server and wait for the OAuth callback.
///
/// Binds to `127.0.0.1:{port}`, waits for a single GET request containing
/// a `code` query parameter, serves a success page, and returns the
/// extracted parameters.
///
/// # Errors
///
/// Returns an error if:
/// - The port cannot be bound
/// - No connection is received
/// - The request does not contain a `code` parameter
/// - The request contains an `error` parameter (OAuth error response)
pub fn wait_for_callback(port: u16) -> Result<CallbackParams, CallbackError> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .map_err(|e| CallbackError::Transport(eyre!("failed to bind to port {port}: {e}")))?;

    tracing::debug!("OAuth callback server listening on port {port}");

    let (mut stream, _addr) = listener
        .accept()
        .map_err(|e| CallbackError::Transport(eyre!("failed to accept connection: {e}")))?;

    // Read the HTTP request
    let mut buf = [0u8; 4096];
    let n = std::io::Read::read(&mut stream, &mut buf)
        .map_err(|e| CallbackError::Transport(eyre!("failed to read request: {e}")))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Extract the request path from the first line: "GET /path?query HTTP/1.1"
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| CallbackError::Transport(eyre!("empty HTTP request")))?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| CallbackError::Transport(eyre!("malformed HTTP request line")))?;

    // Parse query parameters
    let full_url = format!("http://127.0.0.1:{port}{path}");
    let url = Url::parse(&full_url)
        .map_err(|e| CallbackError::Transport(eyre!("failed to parse callback URL: {e}")))?;

    // Check for OAuth error response
    if let Some(error) = url.query_pairs().find(|(k, _)| k == "error") {
        let description = url
            .query_pairs()
            .find(|(k, _)| k == "error_description")
            .map(|(_, v)| v.to_string())
            .unwrap_or_default();
        // Send error page before returning
        let error_html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Authorization Failed</title></head>
<body style="font-family: sans-serif; text-align: center; padding: 2em;">
<h2>Authorization failed</h2>
<p>{}: {}</p>
</body>
</html>"#,
            error.1, description
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            error_html.len(),
            error_html
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        return Err(CallbackError::OAuthDenied {
            error: error.1.to_string(),
            description,
        });
    }

    // Extract code
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| {
            CallbackError::Transport(eyre!("callback did not contain a 'code' parameter"))
        })?;

    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string());

    // Send success response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        SUCCESS_HTML.len(),
        SUCCESS_HTML
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|e| CallbackError::Transport(eyre!("failed to write response: {e}")))?;
    stream
        .flush()
        .map_err(|e| CallbackError::Transport(eyre!("failed to flush response: {e}")))?;

    Ok(CallbackParams { code, state })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_extracts_code_and_state() -> Result<()> {
        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);

        // Spawn a thread that connects and sends a fake OAuth redirect
        let handle = std::thread::spawn(move || wait_for_callback(port));

        // Give the server a moment to start
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Simulate the browser redirect
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))?;
        let request =
            "GET /?code=test_code_123&state=test_state_456 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        std::io::Write::write_all(&mut client, request.as_bytes())?;

        // Read the response
        let mut response = Vec::new();
        std::io::Read::read_to_end(&mut client, &mut response)?;
        let response_str = String::from_utf8_lossy(&response);
        assert!(response_str.contains("Authorization complete"));

        let params = handle
            .join()
            .map_err(|_| eyre!("thread panicked"))?
            .map_err(|e| eyre!("{e}"))?;
        assert_eq!(params.code, "test_code_123");
        assert_eq!(params.state.as_deref(), Some("test_state_456"));
        Ok(())
    }

    #[test]
    fn callback_handles_oauth_error() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);

        let handle = std::thread::spawn(move || wait_for_callback(port));

        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))?;
        let request = "GET /?error=access_denied&error_description=User+denied+access HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        std::io::Write::write_all(&mut client, request.as_bytes())?;

        let mut response = Vec::new();
        std::io::Read::read_to_end(&mut client, &mut response)?;

        let result = handle.join().map_err(|_| eyre!("thread panicked"))?;
        let err = result.expect_err("expected OAuth error");
        assert!(matches!(err, CallbackError::OAuthDenied { .. }));
        assert!(err.to_string().contains("access_denied"));
        Ok(())
    }

    #[test]
    fn callback_without_code_fails() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);

        let handle = std::thread::spawn(move || wait_for_callback(port));

        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))?;
        let request = "GET /?state=only_state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        std::io::Write::write_all(&mut client, request.as_bytes())?;

        // Read response to avoid connection reset
        let mut response = Vec::new();
        let _ = std::io::Read::read_to_end(&mut client, &mut response);

        let result = handle.join().map_err(|_| eyre!("thread panicked"))?;
        assert!(matches!(result, Err(CallbackError::Transport(_))));
        Ok(())
    }
}
