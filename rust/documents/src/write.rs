use std::{
    path::Path,
    time::{Duration, Instant},
};

use common::{itertools::Itertools, tokio, tracing};

use crate::{
    document::{
        Document, DocumentLastWritten, DocumentResponseSender, DocumentRoot,
        DocumentWriteRequestReceiver,
    },
    messages::send_responses,
    When,
};

impl Document {
    /// A background task to write the document to its path on request
    ///
    /// # Arguments
    ///
    /// - `id`: The id of the document
    ///
    /// - `path`: The filesystem path to write to
    ///
    /// - `format`: The format to write (defaults to the path extension)
    ///
    /// - `root`: The root [`Node`] to write (will be read locked)
    ///
    /// - `last_written`: The instant the document was last written (will be write locaked)
    ///
    /// - `request_receiver`: The channel to receive [`WriteRequest`]s on
    ///
    /// - `response_sender`: The channel to send a [`Response`] on when each request if fulfilled
    pub(crate) async fn write_task(
        id: &str,
        path: &Path,
        format: Option<&str>,
        root: &DocumentRoot,
        last_written: &DocumentLastWritten,
        request_receiver: &mut DocumentWriteRequestReceiver,
        response_sender: &DocumentResponseSender,
    ) {
        let duration = Duration::from_millis(Document::WRITE_DEBOUNCE_MILLIS);
        let mut request_ids = Vec::new();
        loop {
            match tokio::time::timeout(duration, request_receiver.recv()).await {
                // Request received: record and continue to wait for timeout unless `when` is now
                Ok(Some(mut request)) => {
                    if !matches!(request.when, When::Never) {
                        request_ids.append(&mut request.ids);
                        if !matches!(request.when, When::Now) {
                            continue;
                        }
                    }
                }
                // Sender dropped: end of task
                Ok(None) => break,
                // Timeout so do the following with the last unhandled request, if any
                Err(..) => {}
            };

            if request_ids.is_empty() {
                continue;
            }

            tracing::trace!(
                "Writing document `{}` to `{}` for requests `{}`",
                id,
                path.display(),
                request_ids.iter().join(",")
            );
            if let Err(error) = codecs::to_path(&*root.read().await, path, format, None).await {
                tracing::error!(
                    "While writing document `{}` to `{}`: {}",
                    id,
                    path.display(),
                    error
                );
            }

            // "End-of-the-line" so send request responses
            send_responses(response_sender, request_ids);

            *last_written.write().await = Instant::now();
            request_ids = Vec::new();
        }
    }
}
