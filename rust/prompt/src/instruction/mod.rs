use kernel_quickjs::kernel::common::itertools::Itertools;
use schema::MessagePart;

use crate::prelude::*;

#[cfg(test)]
mod tests;

#[derive(Default, Trace)]
#[rquickjs::class]

/// The instruction of the current prompt
pub struct Instruction {
    /// The type of the instruction
    #[qjs(get, enumerable, rename = "type")]
    r#type: String,

    /// The message of the instruction
    #[qjs(get, enumerable)]
    message: Option<String>,

    /// The content of the instruction
    #[qjs(get, enumerable)]
    content: Option<String>,
}

impl From<schema::InstructionBlock> for Instruction {
    fn from(value: schema::InstructionBlock) -> Self {
        Self {
            r#type: value.instruction_type.to_string(),
            message: value.message.map(|message| {
                message
                    .parts
                    .into_iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(text.value.to_string()),
                        _ => None,
                    })
                    .join(" ")
            }),
            content: value
                .content
                .map(|blocks| blocks.iter().map(|block| to_markdown(block)).join("")),
        }
    }
}
