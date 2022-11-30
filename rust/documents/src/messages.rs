use std::time::{Duration, Instant};

use common::{
    eyre::{eyre, Result},
    serde::Deserialize,
    strum::{AsRefStr, EnumString},
    tokio, tracing,
};
use graph::{PlanOrdering, PlanScope};
use node_patch::Patch;
use suids::suid_family;

use crate::document::{
    DocumentCancelRequestSender, DocumentCompileRequestSender, DocumentExecuteRequestSender,
    DocumentPatchRequestSender, DocumentResponseReceiver, DocumentResponseSender,
    DocumentWriteRequestSender,
};

/// When requests should be fulfilled by tasks
#[derive(Debug, Default, Clone, Copy, EnumString, Deserialize)]
#[serde(crate = "common::serde")]
#[strum(crate = "common::strum")]
pub enum When {
    // The request should be fulfilled immediately
    #[default]
    Now,

    // The request should be fulfilled on the next delay iteration
    Later,

    // No request needs to be fulfilled
    Never,
}

impl When {
    // Modify a `When` to fulfil the request no later than another `When`
    pub fn no_later_than(&mut self, other: When) {
        if matches!(
            (&self, other),
            (When::Later, When::Now) | (When::Never, When::Later) | (When::Never, When::Now)
        ) {
            *self = other
        }
    }
}

/// A specification of which other tasks should be performed (and when)
/// after a task is performed.
///
/// Use `Then::nothing()` to specify that no subsequent tasks need to
/// be performed:
///
/// ```
/// use documents::Then;
///
/// Then::nothing();
///
/// // Equivalent to
///
/// Then::default();
/// ```
///
/// Use the `compile`, `execute`, etc functions to specify one other
/// task needs to be performed:
///
/// ```
/// use documents::{Then, When};
///
/// Then::compile(When::Later);
/// ```
///
/// Uses `#[serde(default)]` so that clients can send partial JSON objects
/// specifying only one or two tasks (the others defaulting to `Never`).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, crate = "common::serde")]
pub struct Then {
    pub(crate) compile: When,
    pub(crate) execute: When,
    pub(crate) cancel: When,
    pub(crate) write: When,
}

impl Default for Then {
    fn default() -> Self {
        Self {
            compile: When::Never,
            execute: When::Never,
            cancel: When::Never,
            write: When::Never,
        }
    }
}

impl Then {
    /// Create a `Then` that does nothing
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Create a `Then` that compiles `when`
    pub fn compile(when: When) -> Then {
        Self {
            compile: when,
            ..Default::default()
        }
    }

    /// Create a `Then` that executes `when`
    pub fn execute(when: When) -> Then {
        Self {
            execute: when,
            ..Default::default()
        }
    }

    /// Create a `Then` that writes `when`
    pub fn write(when: When) -> Then {
        Self {
            write: when,
            ..Default::default()
        }
    }

    // Modify a `Then` to fulfil each request no later than those in another `Then`
    pub fn no_later_than(&mut self, other: Then) {
        self.compile.no_later_than(other.compile);
        self.execute.no_later_than(other.execute);
        self.cancel.no_later_than(other.cancel);
        self.write.no_later_than(other.write);
    }
}

/// A request to perform a task
#[derive(Debug, AsRefStr)]
#[strum(crate = "common::strum")]
pub enum Request {
    Patch(PatchRequest),
    Compile(CompileRequest),
    Execute(ExecuteRequest),
    Cancel(CancelRequest),
    Write(WriteRequest),
}

pub struct DocumentRequestSenders {
    pub(crate) patch: DocumentPatchRequestSender,
    pub(crate) compile: DocumentCompileRequestSender,
    pub(crate) execute: DocumentExecuteRequestSender,
    pub(crate) cancel: DocumentCancelRequestSender,
    pub(crate) write: DocumentWriteRequestSender,
}

pub async fn send_any_request(request_senders: &DocumentRequestSenders, request: Request) {
    use Request::*;
    match request {
        Patch(request) => request_senders
            .patch
            .send(request)
            .await
            .unwrap_or_else(|error| {
                tracing::error!("While sending patch request: {}", error);
            }),
        Compile(request) => request_senders
            .compile
            .send(request)
            .await
            .unwrap_or_else(|error| {
                tracing::error!("While sending compile request: {}", error);
            }),
        Execute(request) => request_senders
            .execute
            .send(request)
            .await
            .unwrap_or_else(|error| {
                tracing::error!("While sending execute request: {}", error);
            }),
        Cancel(request) => request_senders
            .cancel
            .send(request)
            .await
            .unwrap_or_else(|error| {
                tracing::error!("While sending cancel request: {}", error);
            }),
        Write(request) => request_senders
            .write
            .send(request)
            .await
            .unwrap_or_else(|error| {
                tracing::error!("While sending write request: {}", error);
            }),
    }
}

suid_family!(RequestId, "re");

/// An internal request to patch a document
///
/// At present patch requests do not have a `when` property: all
/// patches are applied immediately.
#[derive(Debug, Default)]
pub struct PatchRequest {
    pub ids: Vec<RequestId>,
    pub patch: Patch,
    pub then: Then,
}

impl PatchRequest {
    /// Get the first id of the request
    pub fn id(&self) -> Result<RequestId> {
        self.ids
            .first()
            .cloned()
            .ok_or_else(|| eyre!("Request has no ids!"))
    }

    /// Create a request to patch a document
    pub fn new(patch: Patch, then: Then) -> Self {
        Self {
            ids: vec![RequestId::new()],
            patch,
            then,
        }
    }

    /// Create a request to patch a document immediately
    ///
    /// An alias for `new` provided for consistency with other requests
    pub fn now(patch: Patch, then: Then) -> Self {
        Self::new(patch, then)
    }

    /// Create a request to patch a document immediately with defaults for options
    pub fn now_defaults(patch: Patch) -> Self {
        Self::now(patch, Then::write(When::Later))
    }

    /// Forward requests to patch a document
    pub fn forward(ids: Vec<RequestId>, patch: Patch, then: Then) -> Self {
        Self { ids, patch, then }
    }
}

/// Sends a [`Patch`] request (if the patch is not empty)
pub async fn send_patch(patch_sender: &DocumentPatchRequestSender, patch: Patch, then: Then) {
    if !patch.is_empty() {
        tracing::trace!(
            "Sending patch request with `{}` operations",
            patch.ops.len()
        );
        if let Err(error) = patch_sender.send(PatchRequest::now(patch, then)).await {
            tracing::error!("While sending patch: {}", error);
        }
    }
}

/// Sends multiple [`Patch`]es using a channel sender (combining them into a single patch, if
/// possible, before sending)
pub async fn send_patches(
    patch_sender: &DocumentPatchRequestSender,
    patches: Vec<Patch>,
    then: Then,
) {
    if patches.iter().any(|patch| patch.target.is_some()) {
        for patch in patches {
            send_patch(patch_sender, patch, then.clone()).await
        }
    } else {
        let patch = Patch::from_patches(patches);
        send_patch(patch_sender, patch, then).await
    }
}

/// An internal request to compile a document
#[derive(Debug)]
pub struct CompileRequest {
    pub ids: Vec<RequestId>,
    pub when: When,
    pub then: Then,
}

impl CompileRequest {
    /// Get the first id of the request
    pub fn id(&self) -> Result<RequestId> {
        self.ids
            .first()
            .cloned()
            .ok_or_else(|| eyre!("Request has no ids!"))
    }

    /// Create a request to compile a document
    pub fn new(when: When, then: Then) -> Self {
        Self {
            ids: vec![RequestId::new()],
            when,
            then,
        }
    }

    /// Create a request to compile a document immediately
    pub fn now(then: Then) -> Self {
        Self::new(When::Now, then)
    }

    /// Create a request to compile a document immediately with defaults for options
    pub fn now_defaults() -> Self {
        Self::now(Then::write(When::Later))
    }

    /// Forward requests to compile a document
    pub fn forward(ids: Vec<RequestId>, when: When, then: Then) -> Self {
        Self { ids, when, then }
    }
}

/// Forward compile requests
#[tracing::instrument(skip(compile_request_sender))]
pub async fn forward_compile_requests(
    compile_request_sender: &DocumentCompileRequestSender,
    request_ids: Vec<RequestId>,
    when: When,
    then: Then,
) {
    tracing::trace!("Forwarding compile requests");
    if let Err(error) = compile_request_sender
        .send(CompileRequest::forward(request_ids, when, then))
        .await
    {
        tracing::error!("While forwarding compile requests: {}", error);
    }
}

/// An internal request to execute a document
#[derive(Debug)]
pub struct ExecuteRequest {
    pub ids: Vec<RequestId>,
    pub when: When,
    pub then: Then,
    pub start: Option<String>,
    pub ordering: Option<PlanOrdering>,
    pub max_concurrency: Option<usize>,
}

impl ExecuteRequest {
    /// Get the first id of the request
    pub fn id(&self) -> Result<RequestId> {
        self.ids
            .first()
            .cloned()
            .ok_or_else(|| eyre!("Request has no ids!"))
    }

    /// Create a request to execute a document
    pub fn new(
        when: When,
        then: Then,
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self {
            ids: vec![RequestId::new()],
            when,
            then,
            start,
            ordering,
            max_concurrency,
        }
    }

    /// Create a request to execute a document immediately
    pub fn now(
        then: Then,
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self::new(When::Now, then, start, ordering, max_concurrency)
    }

    /// Create a request to execute a document with defaults for other options
    pub fn now_defaults() -> Self {
        Self::now(Then::write(When::Later), None, None, None)
    }

    /// Forward requests to execute a document
    pub fn forward(
        ids: Vec<RequestId>,
        when: When,
        then: Then,
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self {
            ids,
            when,
            then,
            start,
            ordering,
            max_concurrency,
        }
    }
}

/// Forward execute requests
#[tracing::instrument(skip(execute_request_sender))]
pub async fn forward_execute_requests(
    execute_request_sender: &DocumentExecuteRequestSender,
    request_ids: Vec<RequestId>,
    when: When,
    then: Then,
) {
    tracing::trace!("Forwarding execute requests");
    if let Err(error) = execute_request_sender
        .send(ExecuteRequest::forward(
            request_ids,
            when,
            then,
            None,
            None,
            None,
        ))
        .await
    {
        tracing::error!("While forwarding execute requests: {}", error);
    }
}

/// An internal request to cancel execution of a document
#[derive(Debug)]
pub struct CancelRequest {
    pub ids: Vec<RequestId>,
    pub start: Option<String>,
    pub scope: Option<PlanScope>,
}

impl CancelRequest {
    /// Get the first id of the request
    pub fn id(&self) -> Result<RequestId> {
        self.ids
            .first()
            .cloned()
            .ok_or_else(|| eyre!("Request has no ids!"))
    }

    /// Create a request to cancel execution of a document immediately
    pub fn now(start: Option<String>, scope: Option<PlanScope>) -> Self {
        Self {
            ids: vec![RequestId::new()],
            start,
            scope,
        }
    }

    /// Create a request to cancel execution of a document
    pub fn forward(ids: Vec<RequestId>, start: Option<String>, scope: Option<PlanScope>) -> Self {
        Self { ids, start, scope }
    }
}

/// An internal request to write the document (e.g. after patching)
#[derive(Debug)]
pub struct WriteRequest {
    pub ids: Vec<RequestId>,
    pub when: When,
}

impl WriteRequest {
    /// Get the first id of the request
    pub fn id(&self) -> Result<RequestId> {
        self.ids
            .first()
            .cloned()
            .ok_or_else(|| eyre!("Request has no ids!"))
    }

    /// Create a request to write a document
    pub fn new(when: When) -> Self {
        Self {
            ids: vec![RequestId::new()],
            when,
        }
    }

    /// Create a request to write a document immediately
    pub fn now() -> Self {
        Self::new(When::Now)
    }

    /// Forward requests to write a document
    pub fn forward(ids: Vec<RequestId>, when: When) -> Self {
        Self { ids, when }
    }
}

/// Forward write requests
#[tracing::instrument(skip(write_request_sender))]
pub async fn forward_write_requests(
    write_request_sender: &DocumentWriteRequestSender,
    request_ids: Vec<RequestId>,
    when: When,
) {
    tracing::trace!("Forwarding write requests");
    if let Err(error) = write_request_sender
        .send(WriteRequest::forward(request_ids.clone(), when))
        .await
    {
        tracing::error!("While forwarding write requests: {}", error);
    }
}

/// A response to a request
#[derive(Debug, Clone)]
pub struct Response {
    pub request_id: RequestId,
}

impl Response {
    pub fn new(request_id: RequestId) -> Self {
        Self { request_id }
    }
}

/// Send responses to requests
#[tracing::instrument(skip(response_sender))]
pub fn send_responses(response_sender: &DocumentResponseSender, request_ids: Vec<RequestId>) {
    for request_id in request_ids {
        if let Err(error) = response_sender.send(Response::new(request_id)) {
            tracing::debug!("While sending request response: {}", error);
        }
    }
}

#[macro_export]
macro_rules! send_request {
    ($sender:expr, $doc_id:expr, $kind:expr, $request:expr) => {{
        let request_id = $request.id()?;

        tracing::debug!(
            "Sending {} request `{}` for document `{}`",
            $kind,
            request_id,
            $doc_id
        );
        if let Err(error) = $sender.send($request).await {
            bail!(
                "When sending {} request for document `{}`: {}",
                $kind,
                $doc_id,
                error
            )
        };

        Ok(request_id)
    }};
}

/// Wait until a response for a request is received
///
/// Warns if the wait takes longer that expected (with exponential backoff
/// to avoid too may warning messages);
pub async fn await_response(
    response_receiver: &mut DocumentResponseReceiver,
    document_id: &str,
    kind: &str,
    request_id: RequestId,
    warning_secs: u64,
) -> Result<()> {
    tracing::trace!(
        "Waiting for response to {} request `{}` for document `{}`",
        kind,
        request_id,
        document_id,
    );

    let start = Instant::now();
    let base: u64 = 2;
    let mut count = 0;

    let warn = || {
        tracing::warn!(
            "Still waiting for response to {} request `{}` for document `{}`; {}s elapsed",
            kind,
            request_id,
            document_id,
            start.elapsed().as_secs()
        );
    };

    loop {
        let last = Instant::now();
        let warning_wait = warning_secs * base.pow(count);
        match tokio::time::timeout(Duration::from_secs(warning_wait), response_receiver.recv())
            .await
        {
            Ok(Ok(response)) => {
                if response.request_id == request_id {
                    tracing::trace!(
                        "Received response to {} request `{}` for document `{}`",
                        kind,
                        document_id,
                        request_id
                    );
                    break;
                }

                if last.elapsed().as_secs() > warning_wait {
                    warn();
                    count += 1;
                }
            }
            Ok(Err(..)) => break,
            Err(..) => {
                warn();
                count += 1;
            }
        }
    }

    Ok(())
}
