use crate::utils::uuids;
use defaults::Defaults;
use eyre::{bail, eyre, Result};
use hmac::Mac;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use zmq::Socket;

pub type HmacSha256 = hmac::Hmac<sha2::Sha256>;

/// The type of a Jupyter message
///
/// This list is from https://jupyter-client.readthedocs.io/en/stable/messaging.html.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum JupyterMessageType {
    // Messages on the shell (ROUTER/DEALER) channel
    execute_request,
    execute_reply,
    inspect_request,
    inspect_reply,
    complete_request,
    complete_reply,
    history_request,
    history_reply,
    is_complete_request,
    is_complete_reply,
    connect_request,
    connect_reply,
    comm_info_request,
    comm_info_reply,
    kernel_info_request,
    kernel_info_reply,
    // Messages on the Control (ROUTER/DEALER) channel
    shutdown_request,
    shutdown_reply,
    interrupt_request,
    interrupt_reply,
    debug_request,
    debug_reply,
    // Messages on the IOPub (PUB/SUB) channel
    stream,
    display_data,
    update_display_data,
    execute_input,
    execute_result,
    error,
    status,
    clear_output,
    debug_event,
    // Messages on the stdin (ROUTER/DEALER) channel
    input_request,
    input_reply,
}

/// The header of a Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#message-header.
/// Note that communication with some kernels may fail if one of more of these fields
/// is missing.
#[derive(Debug, Clone, Defaults, Deserialize, Serialize)]
#[serde(default)]
pub struct JupyterMessageHeader {
    /// The version of the message protocol
    #[def = "\"5.3\".to_string()"]
    pub(crate) version: String,

    /// The type of message
    #[def = "JupyterMessageType::execute_request"]
    pub(crate) msg_type: JupyterMessageType,

    /// A unique identifier for the message
    #[def = "uuids::generate(uuids::Family::Generic)"]
    pub(crate) msg_id: String,

    /// A unique identifier for the kernel session
    pub(crate) session: String,

    /// The name of the user
    ///
    /// We currently leave this blank but it is required by some kernels (e.g. `IJulia`)
    pub(crate) username: String,

    /// ISO 8601 timestamp for when the message was created
    #[def = "chrono::Utc::now().to_rfc3339()"]
    pub(crate) date: String,
}

impl JupyterMessageHeader {
    /// Create a new message header
    fn new(msg_type: JupyterMessageType) -> Self {
        JupyterMessageHeader {
            msg_type,
            ..Default::default()
        }
    }
}

// Each message type has its own structure to the message `content`.
// The following content type definitions implement some of those structures
// on an as needed bases to reduce the need to use lots of `get("...")` calls
// on `serde_json::Value` (the default content type). Note that,
// for both convenience and robustness, `serde_json::Value` is still used for
// some fields in these structs.
//
// Those definitions, including comments, are taken from
// https://jupyter-client.readthedocs.io/en/stable/messaging.html.

/// Content of a `kernel_info_reply` message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#kernel-info
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
pub struct JupyterKernelInfoReply {
    /// 'ok' if the request succeeded or 'error', with error information as in all other replies.
    pub(crate) status: String,

    /// Version of messaging protocol.
    /// The first integer indicates major version.  It is incremented when
    /// there is any backward incompatible change.
    /// The second integer indicates minor version.  It is incremented when
    /// there is any backward compatible change.
    pub(crate) protocol_version: String,

    /// The kernel implementation name
    /// (e.g. 'ipython' for the IPython kernel)
    pub(crate) implementation: String,

    /// Implementation version number.
    /// The version number of the kernel's implementation
    /// (e.g. IPython.__version__ for the IPython kernel)
    pub(crate) implementation_version: String,

    /// Information about the language of code for the kernel
    pub(crate) language_info: serde_json::Value,

    /// A banner of information about the kernel,
    /// which may be displayed in console environments.
    pub(crate) banner: String,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    pub(crate) debugger: bool,

    /// Optional: A list of dictionaries, each with keys 'text' and 'url'.
    /// These will be displayed in the help menu in the notebook UI.
    pub(crate) help_links: serde_json::Value,
}

/// Content of an `execute_request` message
#[derive(Debug, Defaults, Serialize)]
#[serde(default)]
pub struct JupyterExecuteRequest {
    // Source code to be executed by the kernel, one or more lines.
    pub(crate) code: String,

    // A boolean flag which, if True, signals the kernel to execute
    // this code as quietly as possible.
    // silent=True forces store_history to be False,
    // and will *not*:
    //   - broadcast output on the IOPUB channel
    //   - have an execute_result
    // The default is False.
    #[def = "false"]
    pub(crate) silent: bool,

    // A boolean flag which, if True, signals the kernel to populate history
    // The default is True if silent is False.  If silent is True, store_history
    // is forced to be False.
    #[def = "true"]
    pub(crate) store_history: bool,

    // A dict mapping names to expressions to be evaluated in the
    // user's dict. The rich display-data representation of each will be evaluated after execution.
    // See the display_data content for the structure of the representation data.
    #[def = "json!({})"]
    pub(crate) user_expressions: serde_json::Value,

    // Some frontends do not support stdin requests.
    // If this is true, code running in the kernel can prompt the user for input
    // with an input_request message (see below). If it is false, the kernel
    // should not send these messages.
    #[def = "false"]
    pub(crate) allow_stdin: bool,

    // A boolean flag, which, if True, aborts the execution queue if an exception is encountered.
    // If False, queued execute_requests will execute even if this request generates an exception.
    #[def = "false"]
    pub(crate) stop_on_error: bool,
}

/// Content of a `display_data` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
pub struct JupyterDisplayData {
    /// The data dict contains key/value pairs, where the keys are MIME
    /// types and the values are the raw data of the representation in that
    /// format.
    pub(crate) data: HashMap<String, serde_json::Value>,

    /// Any metadata that describes the data
    pub(crate) metadata: HashMap<String, serde_json::Value>,

    /// Optional transient data introduced in 5.1. Information not to be
    /// persisted to a notebook or other documents. Intended to live only
    /// during a live kernel session.
    pub(crate) transient: HashMap<String, serde_json::Value>,
}

/// Content of an `execute_result` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
pub struct JupyterExecuteResult {
    // The counter for this execution is also provided so that clients can
    // display it, since IPython automatically creates variables called _N
    // (for prompt N).
    pub(crate) execution_count: u32,

    // `data` and `metadata` are identical to a display_data message.
    // the object being displayed is that passed to the display hook,
    // i.e. the *result* of the execution.
    pub(crate) data: HashMap<String, serde_json::Value>,
    pub(crate) metadata: HashMap<String, serde_json::Value>,
}

/// Content of a `status` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
pub struct JupyterStatus {
    /// When the kernel starts to handle a message, it will enter the 'busy'
    /// state and when it finishes, it will enter the 'idle' state.
    /// The kernel will publish state 'starting' exactly once at process startup.
    pub(crate) execution_state: String,
}

/// A Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#general-message-format.
/// Some of the below documentation is copied from there.
#[derive(Debug, Deserialize, Serialize)]
pub struct JupyterMessage {
    /// ZeroMQ socket identities
    pub(crate) identities: Vec<String>,

    /// The message header
    ///
    /// "The message header contains information about the message, such as unique identifiers
    /// for the originating session and the actual message id, the type of message, the version
    /// of the Jupyter protocol, and the date the message was created."
    pub(crate) header: JupyterMessageHeader,

    /// The header of the parent message
    ///
    /// "When a message is the “result” of another message, such as a side-effect (output or status)
    /// or direct reply, the `parent_header` is a copy of the `header` of the message that “caused”
    /// the current message. `_reply` messages MUST have a `parent_header`, and side-effects typically
    /// have a parent. If there is no parent, an empty dict should be used. This parent is used by
    /// clients to route message handling to the right place, such as outputs to a cell.""
    pub(crate) parent_header: Option<JupyterMessageHeader>,

    /// Metadata about the message
    ///
    /// "The metadata dict contains information about the message that is not part of the content.
    /// This is not often used, but can be an extra location to store information about requests and
    /// replies, such as extensions adding information about request or execution context.""
    pub(crate) metadata: serde_json::Value,

    /// The content of the message
    ///
    /// "The content dict is the body of the message. Its structure is dictated by the `msg_type`
    /// field in the header, described in detail for each message below."
    pub(crate) content: serde_json::Value,
}

const DELIMITER: &[u8] = b"<IDS|MSG>";

/// A Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#the-wire-protocol
impl JupyterMessage {
    /// Create a new message
    pub fn new<Content: Serialize>(msg_type: JupyterMessageType, content: Content) -> Self {
        Self {
            identities: Vec::new(),
            header: JupyterMessageHeader::new(msg_type),
            parent_header: None,
            metadata: json!({}),
            content: serde_json::to_value(content).expect("Unable to serialize to a value"),
        }
    }

    /// Create an `kernel_info_request` message
    ///
    /// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#kernel-info
    pub fn kernel_info_request() -> Self {
        Self::new(JupyterMessageType::kernel_info_request, json!({}))
    }

    /// Create an `execute_request` message
    pub fn execute_request(code: &str) -> Self {
        Self::new(
            JupyterMessageType::execute_request,
            JupyterExecuteRequest {
                code: code.to_string(),
                ..Default::default()
            },
        )
    }

    /// Send the message
    pub fn send(&self, session: &str, hmac: &HmacSha256, socket: &Socket) -> Result<()> {
        let mut parts: Vec<&[u8]> = Vec::with_capacity(7);

        for part in &self.identities {
            parts.push(part.as_bytes());
        }

        parts.push(DELIMITER);

        let mut header = self.header.clone();
        header.session = session.to_string();
        let header = serde_json::to_string(&header)?;
        let header = header.as_bytes();

        // "If there is no parent, an empty dict should be used"
        let parent_header = match &self.parent_header {
            Some(header) => serde_json::to_string(header)?,
            None => "{}".to_string(),
        };
        let parent_header = parent_header.as_bytes();

        let metadata = serde_json::to_string(&self.metadata)?;
        let metadata = metadata.as_bytes();

        let content = serde_json::to_string(&self.content)?;
        let content = content.as_bytes();

        let mut hmac = hmac.clone();
        hmac.update(header);
        hmac.update(parent_header);
        hmac.update(metadata);
        hmac.update(content);
        let output = hmac.finalize();
        let hmac = hex::encode(output.into_bytes().as_slice());
        parts.push(hmac.as_bytes());

        parts.push(header);
        parts.push(parent_header);
        parts.push(metadata);
        parts.push(content);

        socket.send_multipart(&parts, 0)?;

        Ok(())
    }

    /// Receive a message
    pub fn receive(hmac: &HmacSha256, socket: &Socket) -> Result<Self> {
        let parts = socket.recv_multipart(0)?;

        let delimiter = parts
            .iter()
            .position(|part| &part[..] == DELIMITER)
            .ok_or_else(|| eyre!("Message is missing delimiter"))?;

        let identities = parts[..delimiter]
            .iter()
            .map(|identity| String::from_utf8_lossy(identity).to_string())
            .collect();

        if parts.len() < delimiter + 5 {
            bail!("Message does not have enough parts")
        }
        let msg_hmac = &parts[delimiter + 1];
        let header = &parts[delimiter + 2];
        let parent_header = &parts[delimiter + 3];
        let metadata = &parts[delimiter + 4];
        let content = &parts[delimiter + 5];

        let mut hmac = hmac.clone();
        hmac.update(header);
        hmac.update(parent_header);
        hmac.update(metadata);
        hmac.update(content);
        if let Err(error) = hmac.verify(&hex::decode(&msg_hmac)?) {
            bail!("Unable to verify message HMAC: {}", error);
        }

        let header = serde_json::from_slice(header)?;
        let parent_header = serde_json::from_slice(parent_header)?;
        let metadata = serde_json::from_slice(metadata)?;
        let content = serde_json::from_slice(content)?;

        Ok(Self {
            identities,
            header,
            parent_header,
            metadata,
            content,
        })
    }

    /// Get the content of a message as a particular type
    pub fn content<Content: DeserializeOwned>(self) -> Result<Content> {
        let content = serde_json::from_value(self.content)?;
        Ok(content)
    }
}
