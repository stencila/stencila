use std::sync::Arc;

use common::{eyre::Result, indexmap::IndexMap, tokio::sync::RwLock};
use kernels::Kernels;
use prompts::{Context, PromptInstance};
use schema::{InstructionType, Object};

use crate::{prelude::*, Phase};

/**
 * Execute a prompt.
 *
 * This is not `impl Executable for Assistant` because we need to pass through
 * additional information such as the instruction type and content. Also
 * allows us to have a fallible function and any error to be attached to the
 * calling instruction.
 *
 * Creates a new set of kernels and an executor (without a patch sender) so that
 * the kernels of the primary executor are not polluted.
 */
pub async fn execute_prompt(
    prompt: &mut PromptInstance,
    instruction_type: &InstructionType,
    content: Option<String>,
    context: &Context,
) -> Result<()> {
    let home = prompt.home();

    let mut kernels = Kernels::new(&home);

    let kernels = Arc::new(RwLock::new(kernels));
    // This should be home of the prompt!
    let mut executor = Executor::new(home, kernels, None, None, None);

    executor.phase = Phase::ExecuteWithoutPatches;
    prompt.content.walk_async(&mut executor).await?;

    Ok(())
}

/**
 * Construct an `instruction` object to execute an prompt against
 */
fn instruction(instruction_type: &InstructionType, content: Option<String>) -> Node {
    let mut object = IndexMap::new();

    object.insert(
        "type".to_string(),
        Primitive::String(instruction_type.to_string()),
    );

    if let Some(content) = content {
        object.insert("content".to_string(), Primitive::String(content));
    }

    Node::Object(Object(object))
}
