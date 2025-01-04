//! Handling of completion related messages
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use std::{ops::Deref, sync::Arc};

use async_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
        CompletionResponse, CompletionTriggerKind, Documentation, InsertTextFormat, MarkupContent,
        MarkupKind,
    },
    ResponseError,
};

use codecs::Positions;
use common::tokio::sync::RwLock;
use schema::{InstructionType, Prompt, StringOrNumber};

use crate::utils::position_to_position16;

pub(super) async fn request(
    params: CompletionParams,
    source: Option<Arc<RwLock<String>>>,
) -> Result<Option<CompletionResponse>, ResponseError> {
    eprintln!("COMPLETION: {params:?}");

    // Get the trigger for the completion
    let trigger_kind = params
        .context
        .as_ref()
        .map(|context| context.trigger_kind)
        .unwrap_or(CompletionTriggerKind::INVOKED);
    let trigger_character = params
        .context
        .and_then(|context| context.trigger_character)
        .unwrap_or_default();

    // Unable to proceed if no source available
    let Some(source) = source else {
        return Ok(None);
    };
    let source = source.read().await;

    eprintln!("SOURCE: '{source}'");

    // Get the source before the cursor (up to start of line)
    let position = position_to_position16(params.text_document_position.position);
    let positions = Positions::new(&source);
    let end = positions
        .index_at_position16(position)
        .unwrap_or(source.len());
    let start = source[..end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let take = end - start;
    let line: String = source.chars().skip(start).take(take).collect();

    eprintln!("LINE: '{line}'");

    // Prompt completions
    if line.starts_with(":::") && line.ends_with('@')
        || (trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER && trigger_character == "@")
    {
        return prompt_completion(&line).await;
    }

    smd_snippets(&line)
}

/// Provide completion list for prompts of an instruction
async fn prompt_completion(before: &str) -> Result<Option<CompletionResponse>, ResponseError> {
    let instruction_type = if before.contains("create ") {
        Some(InstructionType::Create)
    } else if before.contains("edit ") {
        Some(InstructionType::Edit)
    } else if before.contains("fix ") {
        Some(InstructionType::Fix)
    } else if before.contains("describe ") {
        Some(InstructionType::Describe)
    } else {
        None
    };

    let items = prompts::list()
        .await
        .iter()
        .filter_map(|prompt| {
            let Prompt {
                id: Some(id),
                name,
                version,
                description,
                instruction_types,
                ..
            } = prompt.deref()
            else {
                return None;
            };

            if let Some(instruction_type) = &instruction_type {
                if !instruction_types.contains(instruction_type) {
                    return None;
                }
            }

            // This attempts to maintain consistency with the symbols used for
            // `DocumentSymbols` for various node types
            let kind = if id.contains("code") {
                CompletionItemKind::EVENT
            } else if id.contains("math") {
                CompletionItemKind::OPERATOR
            } else if id.contains("styled") {
                CompletionItemKind::COLOR
            } else if id.contains("table") {
                CompletionItemKind::STRUCT
            } else if id.contains("block") {
                CompletionItemKind::CONSTRUCTOR
            } else {
                CompletionItemKind::INTERFACE
            };

            let label = prompts::shorten(id, &instruction_type);

            let version = match version {
                StringOrNumber::String(version) => version.to_string(),
                StringOrNumber::Number(version) => version.to_string(),
            };

            let detail = Some(name.to_string());

            let documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("{description}\n\n{id} v{version}"),
            }));

            Some(CompletionItem {
                kind: Some(kind),
                label,
                detail,
                documentation,
                ..Default::default()
            })
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}

/// Provide list of snippets
///
/// This is a better alternative to providing a `snippets.json` file with VSCode
/// extension because:
///
/// 1. Available to other LSP clients
/// 2. Can trigger on non-alphanumeric chars (e.g. ':')
/// 3. Can suggest snippet only if at start of line
/// 4. Can provide better, richer documentation
fn smd_snippets(line: &str) -> Result<Option<CompletionResponse>, ResponseError> {
    const ITEMS: &[(&str, &str, &str, &str, &str)] = &[
        (
            "::: figure ",
            "::: figure ${1:label}\n\n${2:caption}\n\n$0\n\n:::",
            "Figure with a label and/or caption",
            "Figure",
            "Insert a `Figure` block, optionally with a label and caption,

```smd
::: figure 1

A caption for the image.

![](./image.png)

:::
```
",
        ),
        (
            "::: table ",
            "::: table ${1:label}\n\n${2:caption}\n\n$0\n\n:::",
            "Table with a label and/or caption",
            "Table",
            "Insert a `Table` block, optionally with a label and caption, e.g.

```smd
::: table 1

A caption for the table.

| Year | Count |
...

:::
```
",
        ),
        (
            "::: include ",
            "::: include ${1:source}",
            "Include content from another document",
            "Include",
            "Insert an `IncludeBlock` to include content from another file, e.g.

```smd
::: include some/other/file.md
```
"
        ),
        (
            "::: create ",
            "::: create ${0} :::",
            "AI command to create new content",
            "AI Command: Create",
            "Insert an AI command to create new content, e.g.
            
```smd
::: create code to summarize data
```          
",
        ),
        (
            "::: edit ",
            "::: edit ${0} >>>",
            "AI command to edit existing content",
            "AI Command: Edit",
            "Insert an AI command to edit existing content, e.g.
            
```smd
::: edit more concise >>>

An existing paragraph
```
",
        ),
        (
            "::: fix ",
            "::: fix ${0} >>>",
            "AI command to fix code or math",
            "AI Command: Fix",
            "Insert an AI command to fix code or math that has an error, e.g.
            
````smd
::: fix >>>

```python exec
err!
```
````
",
        ),
        (
            "::: describe ",
            "::: describe ${0} :::",
            "AI command to describe other content",
            "AI Command: Describe",
            "Insert an AI command to describe other content, e.g.
            
```smd
::: describe next table :::
```
",
        ),
        (
            "::: for ",
            "::: for ${1:var} in ${2:expr}\n\n$0\n\n:::",
            "Repeat a block of content",
            "For Block",
            "Content will be repeated for each value of variable in the expression, e.g.

```smd
::: for var in expr

Content to be repeated.

:::
```
",
        ),
        (
            "::: if ",
            "::: if ${1:expr}\n\n$0\n\n:::",
            "Conditionally activate content",
            "If Block",
            "Content will only be shown (and executed) if the expression evaluates to a truthy value,

```smd
::: if expr

Content to be conditionally activated.

:::
```
",
        ),
    ];

    let items = ITEMS
        .iter()
        .filter_map(|&(prefix, body, desc, heading, docs)| {
            (line.is_empty() || prefix.starts_with(line)).then_some(CompletionItem {
                label: prefix.into(),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(desc.into()),
                    ..Default::default()
                }),
                detail: Some(heading.into()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: ["**", desc, "**\n\n", docs].concat(),
                })),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                insert_text: body.strip_prefix(line).map(String::from),
                ..Default::default()
            })
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}
