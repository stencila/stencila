use std::sync::atomic::Ordering;

use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    tracing,
};
use events::{has_subscribers, publish};
use node_patch::{apply, Patch};

use crate::{
    document::{
        Document, DocumentCompileRequestSender, DocumentPatchRequestReceiver,
        DocumentResponseSender, DocumentRoot, DocumentVersion, DocumentWriteRequestSender,
    },
    messages::{CompileRequest, PatchRequest, RequestId, Response, WriteRequest},
    When,
};

impl Document {
    /// Patch the document
    #[tracing::instrument(skip(self))]
    pub async fn patch(
        &mut self,
        patch: Patch,
        compile: When,
        execute: When,
        write: When,
    ) -> Result<()> {
        let request_id = self.patch_request(patch, compile, execute, write).await?;

        tracing::trace!(
            "Waiting for patch response for document `{}` for request `{}`",
            self.id,
            request_id
        );
        while let Ok(response) = self.response_receiver.recv().await {
            if response.request_id == request_id {
                tracing::trace!(
                    "Received patch response for document `{}` for request `{}`",
                    self.id,
                    request_id
                );
                break;
            }
        }

        Ok(())
    }

    /// Request that a [`Patch`] be applied to the root node of the document
    ///
    /// # Arguments
    ///
    /// - `patch`: The patch to apply
    ///
    /// - `compile`: Should the document be compiled after the patch is applied?
    ///
    /// - `execute`: Should the document be executed after the patch is applied?
    ///              If the patch as a `target` then the document will be executed from that
    ///              node, otherwise the entire document will be executed.
    /// - `write`: Should the document be written after the patch is applied?
    #[tracing::instrument(skip(self, patch))]
    pub async fn patch_request(
        &self,
        patch: Patch,
        compile: When,
        execute: When,
        write: When,
    ) -> Result<RequestId> {
        tracing::debug!("Sending patch request for document `{}`", self.id);

        let request_id = RequestId::new();
        let request = PatchRequest::new(
            vec![request_id.clone()],
            patch,
            When::Now,
            compile,
            execute,
            write,
        );
        if let Err(error) = self.patch_request_sender.send(request) {
            bail!(
                "When sending patch request for document `{}`: {}",
                self.id,
                error
            )
        };

        Ok(request_id)
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
    #[allow(clippy::too_many_arguments)]
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

            let mut patch = request.patch;
            let start = patch.target.clone();

            // If the patch is empty then continue early rather than obtain locks etc
            if patch.is_empty() {
                continue;
            }

            // Block to ensure locks are retained for only as long as needed
            {
                let root = &mut *root.write().await;

                // Apply the patch to the root node
                if let Err(error) = apply(root, &patch) {
                    tracing::error!("While patching document `{}`: {}", id, error);
                }

                // Increment version (fetch_add returns the previous value, so add one to it)
                let version = version.fetch_add(1, Ordering::Release) + 1;

                // Publish the patch if there are any subscriptions to the `patched` event of this document
                let should_publish = has_subscribers(&topic).unwrap_or_else(|error| {
                    tracing::error!("While checking for any subscribers: {}", error);
                    true
                });
                if should_publish {
                    patch.prepublish(version, root);
                    publish(&topic, &patch);
                } else {
                    tracing::trace!(
                        "Document `{}` skipping publishing patch because there are no subscribers",
                        id
                    );
                }
            }

            // Possibly compile, execute, and/or write; or respond
            if !matches!(request.compile, When::Never) {
                tracing::trace!(
                    "Sending compile request for document `{}` for patch requests `{}`",
                    &id,
                    request.ids.iter().join(",")
                );
                if let Err(error) = compile_sender
                    .send(CompileRequest::new(
                        request.ids,
                        request.compile,
                        request.execute,
                        request.write,
                        start,
                    ))
                    .await
                {
                    tracing::error!(
                        "While sending compile request for document `{}`: {}",
                        id,
                        error
                    );
                }
            } else if !matches!(request.write, When::Never) {
                tracing::trace!(
                    "Sending write request for document `{}` for patch requests `{}`",
                    &id,
                    request.ids.iter().join(",")
                );
                if let Err(error) = write_sender.send(WriteRequest::new(request.ids, request.write))
                {
                    tracing::error!(
                        "While sending write request for document `{}`: {}",
                        id,
                        error
                    );
                }
            } else {
                for request_id in request.ids {
                    if let Err(error) = response_sender.send(Response::new(request_id)) {
                        tracing::debug!(
                            "While sending response for document `{}` from patch task: {}",
                            id,
                            error
                        );
                    }
                }
            }
        }
    }
}
