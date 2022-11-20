use std::path::Path;

pub use common::{async_trait::async_trait, eyre::Result, tracing};
pub use node_address::{Address, Slot};
pub use node_patch::{produce_address, diff_address};
pub use stencila_schema::*;

use hash_utils::str_seahash;
use suids::Suid;

pub use super::{CompileContext, Executable, ExecuteContext};

/// Generate a unique id for a node
pub fn generate_id(prefix: &str) -> Option<Suid> {
    Some(suids::generate(prefix))
}

/// Generate a digest from a string
pub fn generate_digest(content: &str) -> u64 {
    str_seahash(&content.replace('\r', "")).unwrap_or_default()
}

/// Get the state_digest of a node's [`ExecutionDigest`]
pub fn get_state_digest(execution_digest: &Option<ExecutionDigest>) -> u64 {
    execution_digest
        .as_ref()
        .map(|compile_digest| compile_digest.state_digest)
        .unwrap_or_default()
}
