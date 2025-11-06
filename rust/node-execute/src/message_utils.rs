use common::eyre::Result;
use schema::{InstructionMessage, MessagePart, Node, Text};

use crate::prelude::*;

/// Check if a string contains Jinja template syntax
fn contains_jinja_syntax(text: &str) -> bool {
    text.contains("{{") && text.contains("}}")
}

/// Render a text string through Jinja kernel to resolve variables
async fn render_text_through_jinja(
    text: &str,
    executor: &mut Executor,
) -> Result<String> {
    // Create or get a Jinja kernel instance
    let mut kernels = executor.kernels().await;
    let jinja_instance = kernels.create_instance(Some("jinja")).await?;
    drop(kernels); // Release the lock before awaiting

    // Execute the text as a Jinja template
    let mut instance = jinja_instance.lock().await;
    let (outputs, messages) = instance.execute(text).await?;

    // Check for execution errors
    if let Some(error_msg) = messages
        .iter()
        .find(|msg| matches!(msg.level, schema::MessageLevel::Error | schema::MessageLevel::Exception))
    {
        tracing::warn!("Jinja rendering error: {}", error_msg.message);
        // Return original text if rendering fails
        return Ok(text.to_string());
    }

    // Extract the rendered string from outputs
    // The Jinja kernel returns a Node::String with the rendered text
    if let Some(Node::String(rendered)) = outputs.first() {
        Ok(rendered.to_string())
    } else {
        // If no output or unexpected format, return original text
        tracing::warn!("Jinja kernel returned unexpected output format: {:?}", outputs);
        Ok(text.to_string())
    }
}

/// Render variables in an InstructionMessage through Jinja
///
/// This function processes each text part of the message and renders
/// any Jinja templates (e.g., `{{variable}}`) through a Jinja kernel
/// to resolve variables from code kernels.
pub async fn render_message_variables(
    message: &InstructionMessage,
    executor: &mut Executor,
) -> Result<InstructionMessage> {
    let mut rendered_parts = Vec::new();

    for part in &message.parts {
        match part {
            MessagePart::Text(text) => {
                let text_value = text.value.to_string();
                
                // Only render if the text contains Jinja syntax
                if contains_jinja_syntax(&text_value) {
                    let rendered = render_text_through_jinja(&text_value, executor).await?;
                    rendered_parts.push(MessagePart::Text(Text::from(rendered)));
                } else {
                    // No Jinja syntax, keep original
                    rendered_parts.push(part.clone());
                }
            }
            _ => {
                // Preserve non-text parts (images, audio, video) as-is
                rendered_parts.push(part.clone());
            }
        }
    }

    // Create new message with rendered parts
    Ok(InstructionMessage {
        parts: rendered_parts,
        ..message.clone()
    })
}

