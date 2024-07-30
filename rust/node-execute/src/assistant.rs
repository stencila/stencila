use std::{path::Path, sync::Arc};

use common::{eyre::Result, indexmap::IndexMap, tokio::sync::RwLock};
use kernels::Kernels;
use schema::{Assistant, InstructionType, Object};

use crate::{prelude::*, Phase};

/**
 * Execute an assistant.
 *
 * This is not `impl Executable for Assistant` because we need to pass through
 * additional information such as the instruction type and content. Also 
 * allows us to have a fallible function and any error to be attached to the
 * calling instruction.
 *
 * Creates a new set of kernels and an executor (without a patch sender) so that
 * the kernels of the primary executor are not polluted.
 */
pub async fn execute_assistant(
    assistant: &mut Assistant,
    instruction_type: &InstructionType,
    content: Option<String>,
    home: &Path,
) -> Result<()> {
    let mut kernels = Kernels::new(&home);
    kernels.create_instance(Some("quickjs")).await?;
    kernels
        .set("instruction", &instruction(instruction_type, content))
        .await?;

    let kernels = Arc::new(RwLock::new(kernels));
    let mut executor = Executor::new(home.to_path_buf(), kernels, None, None, None);

    executor.phase = Phase::ExecuteOnly;
    assistant.content.walk_async(&mut executor).await?;

    Ok(())
}

/**
 * Construct a `instruction` object to execute an assistant against
 */
fn instruction(instruction_type: &InstructionType, content: Option<String>) -> Node {
    let mut map = IndexMap::new();

    map.insert(
        "type".to_string(),
        Primitive::String(instruction_type.to_string()),
    );

    if let Some(content) = content {
        map.insert("content".to_string(), Primitive::String(content));
    }

    Node::Object(Object(map))
}
