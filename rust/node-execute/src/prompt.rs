use std::{ops::Deref, sync::Arc};

use common::{
    eyre::{OptionExt, Result},
    itertools::Itertools,
    tokio::sync::RwLock,
};
use kernels::Kernels;
use prompts::prompt::{KernelsContext, PromptContext};
use schema::{AuthorRole, InstructionBlock};

use crate::{prelude::*, Phase};

/**
 * Select and execute a prompt for an [`InstructionBlock`].
 *
 * This is not a `impl Executable for Prompt` because we need to pass through
 * additional information such as the instruction type and content. This also
 * allows us to have a fallible function and any error to be attached to the
 * calling instruction.
 *
 * Creates a new set of kernels and an executor (without a patch sender) so that
 * the kernels of the primary executor are not polluted.
 */
pub async fn for_instruction_block(
    instruction: &InstructionBlock,
    executor: &Executor,
) -> Result<(AuthorRole, String)> {
    // Determine the types of nodes in the content of the instruction
    let node_types = instruction
        .content
        .iter()
        .flatten()
        .map(|block| block.node_type().to_string())
        .collect_vec();

    // Select the best prompt for the instruction
    let mut prompt = prompts::select(
        &instruction.instruction_type,
        &instruction.message,
        &instruction.assignee,
        &Some(node_types),
    )
    .await?;

    // Create an author role for the prompt
    let prompter = AuthorRole {
        last_modified: Some(Timestamp::now()),
        ..prompt.deref().clone().into()
    };

    // Create a prompt context
    // TODO: allow prompts to specify whether they need various parts of context
    // as an optimization, particularly to avoid getting kernel contexts unnecessarily.
    let context = PromptContext {
        instruction: Some(instruction.into()),
        document: Some(executor.document_context.clone()),
        kernels: Some(KernelsContext::from_kernels(executor.kernels.read().await.deref()).await?),
    };

    // Get the home dir of the prompt for instantiating both the kernels and executor
    let home = prompt.home();

    // Create a new kernel instance for the prompt context
    let kernel = kernels::get("quickjs")
        .await
        .ok_or_eyre("QuickJS kernel not available")?;
    let kernel_instance = context.into_kernel().await?;

    // Create a set of kernels to execute the prompt and add the kernel instance to it
    let mut kernels = Kernels::new(&home);
    kernels.add_instance(kernel, kernel_instance).await?;

    // Execute the prompt without patching
    let mut executor = Executor::new(home, Arc::new(RwLock::new(kernels)), None, None, None);
    executor.phase = Phase::ExecuteWithoutPatches;
    prompt.content.walk_async(&mut executor).await?;

    // Render the prompt
    let prompt = prompts::render(prompt).await?;

    Ok((prompter, prompt))
}
