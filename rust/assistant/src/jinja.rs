//! Functions used as filters and elsewhere in `minijinja` templates

use std::{
    hash::{Hash, Hasher},
    sync::Mutex,
};

use common::{
    eyre::{eyre, Report, Result},
    minijinja::{value::ViaDeserialize, Environment, Error, UndefinedBehavior, Value},
    once_cell::sync::Lazy,
    seahash::SeaHasher,
    serde_json, serde_yaml,
};
use schema::{
    ArrayHint, Block, DatatableHint, Hint, InsertBlock, InstructionBlock, MathBlock, MessagePart,
    Node, SuggestionBlockType, SuggestionStatus, Variable,
};

use crate::GenerateTask;

/// Render a prompt with the task as context
pub fn render_template(prompt: &str, task: &GenerateTask) -> Result<String> {
    let mut hasher = SeaHasher::new();
    prompt.hash(&mut hasher);
    let hash = hasher.finish().to_string();

    let mut env = ENV
        .lock()
        .map_err(|error| eyre!("Unable to lock ENV: {error}"))?;

    let template = match env.get_template(&hash) {
        Ok(template) => template,
        _ => {
            env.add_template_owned(hash.clone(), prompt.to_string())
                .map_err(minijinja_error_to_eyre)?;
            env.get_template(&hash).expect("Should be just added")
        }
    };

    let rendered = template
        .render(task)
        .map_err(minijinja_error_to_eyre)?
        .trim()
        .to_string();

    Ok(rendered)
}

/// A template environment for rendering system prompts
static ENV: Lazy<Mutex<Environment>> = Lazy::new(|| {
    let mut env = Environment::new();

    // Set the most lenient undefined behavior to avoid errors
    // rendering templates for users
    env.set_undefined_behavior(UndefinedBehavior::Chainable);

    // Serialization filters
    env.add_filter("to_json", to_json);
    env.add_filter("to_markdown", to_markdown);
    env.add_filter("to_text", to_text);
    env.add_filter("to_yaml", to_yaml);

    // Trimming content filters
    env.add_filter("trim_start_chars", trim_start_chars);
    env.add_filter("trim_end_chars", trim_end_chars);

    // Filters for describing document nodes
    env.add_filter("describe_variable", describe_variable);

    // Filters for providing few-shot examples
    env.add_filter("insert_code_chunk_shots", insert_code_chunk_shots);
    env.add_filter("insert_math_block_shots", insert_math_block_shots);

    Mutex::new(env)
});

/// Expand a `minijinja` error to include the sources of the error (location etc)
fn minijinja_error_to_eyre(error: Error) -> Report {
    let mut error = &error as &dyn std::error::Error;
    let mut message = format!("{error:#}");
    while let Some(source) = error.source() {
        message.push_str(&format!("\n{:#}", source));
        error = source;
    }
    eyre!(message)
}

/// Generate a JSON representation of a Stencila node
fn to_json(node: ViaDeserialize<Node>) -> String {
    serde_json::to_string_pretty(&node.0).unwrap_or_default()
}

/// Generate a Markdown representation of a Stencila node
fn to_markdown(node: ViaDeserialize<Node>) -> String {
    codec_markdown_trait::to_markdown(&node.0)
}

/// Generate a plain text representation of a Stencila node
fn to_text(node: ViaDeserialize<Node>) -> String {
    codec_text_trait::to_text(&node.0)
}

/// Generate a YAML representation of a Stencila node
fn to_yaml(node: ViaDeserialize<Node>) -> String {
    serde_yaml::to_string(&node.0).unwrap_or_default()
}

/// Trim the starting characters from a string so that it is no longer than `length`
fn trim_start_chars(content: &str, length: u32) -> String {
    let current_length = content.chars().count();
    content
        .chars()
        .skip(current_length.saturating_sub(length as usize))
        .take(length as usize)
        .collect()
}

/// Trim the ending characters from a string so that it is no longer than `length`
fn trim_end_chars(content: &str, length: u32) -> String {
    content.chars().take(length as usize).collect()
}

/// Create an Markdown description of a `Variable` as a list item with a
/// nested child list describing its characteristics.
fn describe_variable(variable: ViaDeserialize<Variable>) -> String {
    let mut desc = format!("- Variable `{}`", variable.name);

    if let Some(native_type) = &variable.native_type {
        desc.push_str(" is a ");
        if let Some(programming_language) = &variable.programming_language {
            desc.push_str(programming_language);
            desc.push(' ');
        }
        desc.push_str(&format!("`{native_type}`"));
    }

    if let Some(native_hint) = &variable.native_hint {
        desc.push('\n');
        desc.push_str(native_hint);
        return desc;
    };

    let Some(hint) = &variable.hint else {
        return desc;
    };

    match hint {
        Hint::ArrayHint(hint) => desc += &describe_array_hint(hint),
        Hint::DatatableHint(hint) => desc += &describe_datatable_hint(hint),
        _ => {
            // TODO handle all the other hint types
        }
    }

    desc
}

fn describe_array_hint(hint: &ArrayHint) -> String {
    let mut lines = vec![format!(" with length {}", hint.length)];
    if let Some(item_types) = &hint.item_types {
        lines.push(format!(
            "containing values of the following types: {}",
            item_types.join(",")
        ));
    }
    if let Some(minimum) = &hint.minimum {
        lines.push(format!("with a minimum of: {minimum}"));
    }
    if let Some(maximum) = &hint.maximum {
        lines.push(format!("with a maximum of: {maximum}"));
    }
    if let Some(nulls) = &hint.nulls {
        lines.push(format!("containing {nulls} null values"));
    }
    lines.join("\n    - ")
}

fn describe_datatable_hint(hint: &DatatableHint) -> String {
    let mut header = format!(" with {} rows", hint.rows);
    if hint.columns.is_empty() {
        return header;
    }

    header += ", with these columns:";
    let mut lines = vec![header];
    for column in &hint.columns {
        let mut line = format!("`{}`: type {}", column.name, column.item_type);
        if let Some(minimum) = &column.minimum {
            line.push_str(&format!(", with minimum {minimum}"));
        }
        if let Some(maximum) = &column.maximum {
            line.push_str(&format!(", maximum {maximum}"));
        }
        if let Some(nulls) = &column.nulls {
            line.push_str(&format!(", containing {nulls} null values"));
        }
        lines.push(line);
    }
    lines.join("\n    - ")
}

/// Generate user/assistant pairs of strings for `InstructionBlock`s
/// to render into system prompts for few-shot, in-context learning
fn insert_block_shots<F>(
    instructions: ViaDeserialize<Vec<InstructionBlock>>,
    examples: ViaDeserialize<Vec<(String, String)>>,
    assignee: &str,
    extractor: F,
) -> Result<Value, Error>
where
    F: Fn(Block) -> Option<String>,
{
    let instructions = instructions.0;
    let examples = examples.0;

    let mut shots: Vec<(String, String)> = instructions
        .into_iter()
        .filter_map(|instruction| {
            if instruction.assignee.as_deref() != Some(assignee) {
                return None;
            }

            // Get the first user text instruction. Ignore intermediate user messages involved in refinement.
            let user = instruction
                .messages
                .first()
                .and_then(|message| message.parts.first())
                .and_then(|part| match part {
                    MessagePart::Text(text) => Some(text.to_value_string()),
                    _ => None,
                });

            // Get accepted inserted block
            let block = instruction.suggestion.and_then(|suggestion| {
                if let SuggestionBlockType::InsertBlock(InsertBlock {
                    suggestion_status,
                    mut content,
                    ..
                }) = suggestion
                {
                    if suggestion_status != Some(SuggestionStatus::Accepted) {
                        None
                    } else {
                        (!content.is_empty()).then_some(content.swap_remove(0))
                    }
                } else {
                    None
                }
            });

            // Extract string from the block
            let assistant = match block {
                Some(block) => extractor(block),
                None => None,
            };

            user.zip(assistant)
        })
        .collect();

    // Augment the collected shots with provided examples
    let examples_len = examples.len();
    for example in examples {
        if shots.len() >= examples_len {
            break;
        }
        shots.push(example)
    }

    Ok(Value::from_serialize(&shots))
}

/// Generate example shots for the 'insert-code-chunk' assistant
fn insert_code_chunk_shots(
    instructions: ViaDeserialize<Vec<InstructionBlock>>,
    examples: ViaDeserialize<Vec<(String, String)>>,
) -> Result<Value, Error> {
    insert_block_shots(
        instructions,
        examples,
        "insert-code-chunk",
        |block: Block| match block {
            Block::CodeChunk(block) => Some(block.code.to_string()),
            _ => None,
        },
    )
}

/// Generate example shots for the 'insert-math-block' assistant
fn insert_math_block_shots(
    instructions: ViaDeserialize<Vec<InstructionBlock>>,
    examples: ViaDeserialize<Vec<(String, String)>>,
) -> Result<Value, Error> {
    insert_block_shots(
        instructions,
        examples,
        "insert-math-block",
        |block| match block {
            Block::MathBlock(MathBlock {
                math_language,
                code,
                ..
            }) => {
                let lang = math_language.map(|lang| lang.to_lowercase());
                (lang.is_none()
                    || lang.as_deref() == Some("latex")
                    || lang.as_deref() == Some("tex"))
                .then_some(code.to_string())
            }
            _ => None,
        },
    )
}
