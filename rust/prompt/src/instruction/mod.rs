use kernel_quickjs::kernel::common::{itertools::Itertools, once_cell::sync::Lazy, regex::Regex};
use schema::MessagePart;

use crate::{prelude::*, DocumentContext};

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

    /// The node types of the instruction's content
    #[qjs(get, enumerable)]
    content_types: Option<Vec<String>>,

    /// The content of the instruction as Markdown
    #[qjs(get, enumerable)]
    content: Option<String>,
}

#[rquickjs::methods]
impl Instruction {
    /// Get the target of the instruction as Markdown
    ///
    /// The instruction message is parsed and the target block is resolved.
    #[qjs(get, enumerable)]
    pub fn target<'js>(&self, ctx: Ctx<'js>) -> String {
        let Ok(document) = ctx.globals().get::<_, DocumentContext>("document") else {
            // No document, so just return empty content
            return String::new();
        };

        let Some(message) = &self.message else {
            // No message to be able to resolve target block, so just return the next block
            return document.next_block();
        };

        static TARGET: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"((next)|(prev(ious)?))?\s*((code)|(fig(ure)?)|(tab(le)?))?\s*(\d+)?")
                .expect("invalid regex")
        });
        let Some(captures) = TARGET.captures(message) else {
            // Unable to determine target from message, so just return the next block
            return document.next_block();
        };

        enum TargetPos {
            Previous,
            Next,
        }
        let target_pos = captures.get(1).and_then(|value| match value.as_str() {
            "prev" | "previous" => Some(TargetPos::Previous),
            "next" => Some(TargetPos::Next),
            _ => None,
        });

        enum TargetType {
            Code,
            Figure,
            Table,
        }
        let target_type = captures.get(5).and_then(|value| match value.as_str() {
            "code" => Some(TargetType::Code),
            "fig" | "figure" => Some(TargetType::Figure),
            "tab" | "table" => Some(TargetType::Table),
            _ => None,
        });
        let Some(target_type) = target_type else {
            // No target type so return, previous or next block
            return match target_pos {
                Some(TargetPos::Previous) => document.previous_block(),
                Some(TargetPos::Next) | None => document.next_block(),
            };
        };

        let target_label = captures.get(7).map(|value| value.as_str());
        if let Some(target_label) = target_label {
            for chunk in &document.code_chunks.items {
                let (Some(label_type), Some(label)) = (&chunk.label_type, &chunk.label) else {
                    continue;
                };
                if target_label == label
                    && ((matches!(target_type, TargetType::Figure) && label_type == "Figure")
                        || (matches!(target_type, TargetType::Table) && label_type == "Table"))
                {
                    return chunk.markdown_with_outputs();
                }
            }
        };

        // No targeting keywords found so just return the next block
        document.next_block()
    }

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
            content: value.content.as_ref().map(to_markdown),
        }
    }
}
