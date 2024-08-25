use kernel_quickjs::kernel::common::itertools::Itertools;
use schema::MessagePart;

use crate::prelude::*;

#[cfg(test)]
mod tests;

/// The instruction of the current prompt
#[derive(Default, Trace)]
#[rquickjs::class(rename_all = "camelCase")]
pub struct Instruction {
    /// The type of the instruction
    #[qjs(get, enumerable, rename = "type")]
    r#type: String,

    /// The message of the instruction
    #[qjs(get, enumerable)]
    message: Option<String>,

    /// The node types of the content
    #[qjs(get, enumerable)]
    content_types: Option<Vec<String>>,

    /// The content of the instruction as Markdown
    #[qjs(get, enumerable)]
    content: Option<String>,
}

#[rquickjs::methods]
impl Instruction {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;

        obj.set("type", self.r#type.clone())?;
        obj.set("message", self.message.clone())?;
        obj.set("contentTypes", self.content_types.clone())?;
        obj.set("content", self.content.clone())?;

        Ok(obj)
    }
}

impl From<&schema::InstructionBlock> for Instruction {
    fn from(value: &schema::InstructionBlock) -> Self {
        Self {
            r#type: value.instruction_type.to_string(),
            message: value.message.as_ref().map(|message| {
                message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        MessagePart::Text(text) => Some(text.value.to_string()),
                        _ => None,
                    })
                    .join(" ")
            }),
            content_types: value.content.as_ref().map(|blocks| {
                blocks
                    .iter()
                    .map(|block| block.node_type().to_string())
                    .collect()
            }),
            content: value.content.as_ref().map(|blocks| to_markdown(blocks)),
        }
    }
}
