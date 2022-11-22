use std::sync::atomic::Ordering;

use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    tracing,
};
use events::publish;
use node_patch::{apply, Patch};

use crate::{
    document::{
        Document, DocumentCompileRequestSender, DocumentPatchRequestReceiver,
        DocumentResponseSender, DocumentRoot, DocumentVersion, DocumentWriteRequestSender,
    },
    messages::{
        await_response, forward_compile_requests, forward_write_requests, send_responses,
        PatchRequest, RequestId, Then,
    },
    send_request, When,
};

impl Document {
    /// Patch the document
    #[tracing::instrument(skip(self))]
    pub async fn patch(&mut self, patch: Patch, then: Option<Then>) -> Result<()> {
        let request_id = self.patch_request(patch, then).await?;
        await_response(
            &mut self.response_receiver,
            &self.id,
            "patch",
            request_id,
            1,
        )
        .await
    }

    /// Request that a [`Patch`] be applied to the root node of the document
    ///
    /// # Arguments
    ///
    /// - `patch`: The patch to apply
    #[tracing::instrument(skip(self, patch))]
    pub async fn patch_request(&self, patch: Patch, then: Option<Then>) -> Result<RequestId> {
        // TODO: rather defaulting to compiling, compile only if the node is ops include
        // executable nodes; allow user to specify execute as well after patching executable nodes
        // (i.e. fully reactive).
        let then = then.unwrap_or_else(|| Then {
            compile: When::Later,
            write: When::Later,
            ..Default::default()
        });
        let request = PatchRequest::now(patch, then);
        send_request!(self.patch_request_sender, &self.id, "patch", request)
    }

    /// A background task to patch the root node of the document on request
    ///
    /// Use an unbounded channel for sending patches, so that sending threads never
    /// block (if there are lots of patches) and thereby hold on to locks causing a
    /// deadlock.
    ///
    /// # Arguments
    ///
    /// - `id`: The id of the document (used in the published event topic)
    ///
    /// - `version`: The version of the document (will be incremented after a patch is applied)
    ///
    /// - `root`: The root [`Node`] to apply the patch to (will be write locked)
    ///
    /// - `compile_sender`: The channel to send any [`CompileRequest`]s after a patch is applied
    ///
    /// - `write_sender`: The channel to send any [`WriteRequest`]s after a patch is applied
    ///
    /// - `request_receiver`: The channel to receive [`PatchRequest`]s on
    ///
    /// - `response_sender`: The channel to send a [`Response`] on when each request if fulfilled
    pub(crate) async fn patch_task(
        id: &str,
        version: &DocumentVersion,
        root: &DocumentRoot,
        compile_sender: &DocumentCompileRequestSender,
        write_sender: &DocumentWriteRequestSender,
        request_receiver: &mut DocumentPatchRequestReceiver,
        response_sender: &DocumentResponseSender,
    ) {
        let topic = ["documents:", id, ":patched"].concat();
        while let Some(request) = request_receiver.recv().await {
            tracing::trace!(
                "Patching document `{}` for requests `{}`",
                &id,
                request.ids.iter().join(",")
            );

            // If the patch is empty then continue early rather than obtain locks etc
            let mut patch = request.patch;
            if patch.is_empty() {
                continue;
            }

            // Block to ensure locks are retained for only as long as needed
            {
                let root = &mut *root.write().await;

                // Apply the patch to the root node.
                // Log any errors and skip the rest (don't want to publish an invalid patch
                // or run follow on tasks)
                if let Err(error) = apply(root, &patch) {
                    tracing::error!("While patching document `{}`: {}", id, error);
                    continue;
                }

                // Increment version (fetch_add returns the previous value, so add one to it)
                let version = version.fetch_add(1, Ordering::Release) + 1;

                // Publish the patch
                // Previously we skipped prepublish if there were no subscribers but that seemed to
                // be a premature optimization (and required an extra call to `events::has_subscribers`) so was removed
                patch.prepublish(version, root);
                publish(&topic, &patch);
            }

            // Forward requests or respond
            let PatchRequest { ids, then, .. } = request;
            if !matches!(then.compile, When::Never) {
                forward_compile_requests(compile_sender, ids, then.compile, then).await;
            } else if !matches!(then.write, When::Never) {
                forward_write_requests(write_sender, ids, then.write).await
            } else {
                send_responses(response_sender, ids)
            }
        }
    }
}
