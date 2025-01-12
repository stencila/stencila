//! Handling of completion related messages
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use std::{ops::Deref, sync::Arc};

use async_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
        CompletionResponse, CompletionTextEdit, Documentation, InsertTextFormat, MarkupContent,
        MarkupKind, Position, Range, TextEdit,
    },
    ResponseError,
};

use codecs::Positions;
use common::tokio::sync::RwLock;
use kernels::KernelType;
use schema::{InstructionType, Prompt, StringOrNumber};

use crate::utils::position_to_position16;

pub(super) async fn request(
    params: CompletionParams,
    source: Option<Arc<RwLock<String>>>,
) -> Result<Option<CompletionResponse>, ResponseError> {
    // Unable to proceed if no source available
    let Some(source) = source else {
        return Ok(None);
    };
    let source = source.read().await;

    // Get the source before the cursor (up to start of line)
    let position = params.text_document_position.position;
    let positions = Positions::new(&source);
    let end = positions
        .index_at_position16(position_to_position16(position))
        .unwrap_or(source.len());
    let start = source[..end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let take = end - start;
    let line: String = source.chars().skip(start).take(take).collect();

    // Code chunk completions
    if line.starts_with("```") {
        if line.contains("exec") {
            return execution_keywords(&line);
        } else {
            return kernel_completion().await;
        }
    }

    // Chat and command completions
    if line.starts_with("/") || line.starts_with(":::") {
        if line.ends_with('@') {
            return prompt_completion(&line).await;
        }

        if line.ends_with('[') || (line.contains('[') && !line.contains(']') && line.ends_with(','))
        {
            return model_completion().await;
        }
    }

    // Snippet completions
    smd_snippets(&line, position.line)
}

/// Provide completion list of prompts
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
                name,
                version,
                description,
                instruction_types,
                ..
            } = prompt.deref();

            if let Some(instruction_type) = &instruction_type {
                if !instruction_types.contains(instruction_type) {
                    return None;
                }
            }

            // This attempts to maintain consistency with the symbols used for
            // `DocumentSymbols` for various node types
            let kind = if name.contains("code") {
                CompletionItemKind::EVENT
            } else if name.contains("math") {
                CompletionItemKind::OPERATOR
            } else if name.contains("styled") {
                CompletionItemKind::COLOR
            } else if name.contains("table") {
                CompletionItemKind::STRUCT
            } else if name.contains("block") {
                CompletionItemKind::CONSTRUCTOR
            } else {
                CompletionItemKind::INTERFACE
            };

            let label = prompts::shorten(name, &instruction_type);

            let version = match version {
                StringOrNumber::String(version) => version.to_string(),
                StringOrNumber::Number(version) => version.to_string(),
            };

            let detail = Some(name.to_string());

            let documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("{description}\n\n{name} v{version}"),
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

/// Provide completion list of models
async fn model_completion() -> Result<Option<CompletionResponse>, ResponseError> {
    let items = models::list()
        .await
        .iter()
        .filter_map(|model| {
            if !model.is_available() {
                return None;
            }

            let label = model.id();
            let detail = Some(format!(
                "{} {} {}",
                model.provider(),
                model.name(),
                model.version()
            ));

            Some(CompletionItem {
                label,
                detail,
                ..Default::default()
            })
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}

/// Provide completion list of kernels
async fn kernel_completion() -> Result<Option<CompletionResponse>, ResponseError> {
    let items = kernels::list()
        .await
        .iter()
        .filter_map(|kernel| {
            if !kernel.is_available() {
                return None;
            }

            let kind = match kernel.r#type() {
                KernelType::Programming => CompletionItemKind::EVENT,
                KernelType::Math => CompletionItemKind::OPERATOR,
                KernelType::Diagrams => CompletionItemKind::INTERFACE,
                KernelType::Templating => CompletionItemKind::KEYWORD,
                KernelType::Styling => {
                    return None;
                }
            };

            let mut label = kernel.name();
            if matches!(
                kernel.r#type(),
                KernelType::Programming | KernelType::Diagrams | KernelType::Templating
            ) {
                label.push_str(" exec");
            }

            Some(CompletionItem {
                kind: Some(kind),
                label,
                ..Default::default()
            })
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}

/// Provide list of keyword for execution mode and bounds
fn execution_keywords(line: &str) -> Result<Option<CompletionResponse>, ResponseError> {
    const MODE: [&str; 4] = ["auto", "always", "lock", "need"];
    const BOUNDS: [&str; 5] = ["fork", "limit", "box", "skip", "main"];

    let has_bounds = BOUNDS.iter().any(|word| line.contains(word));
    if has_bounds {
        return Ok(None);
    }

    let mut items = Vec::new();

    // Order as defined above
    let mut order = "abcdefghijklmnop".chars();

    let has_mode = MODE.iter().any(|word| line.contains(word));
    if !has_mode {
        items.append(
            &mut MODE
                .into_iter()
                .map(|mode| CompletionItem {
                    kind: Some(CompletionItemKind::EVENT),
                    label: mode.into(),
                    sort_text: order.next().map(String::from),
                    ..Default::default()
                })
                .collect(),
        );
    }

    items.append(
        &mut BOUNDS
            .into_iter()
            .map(|mode| CompletionItem {
                kind: Some(CompletionItemKind::MODULE),
                label: mode.into(),
                sort_text: order.next().map(String::from),
                ..Default::default()
            })
            .collect(),
    );

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
fn smd_snippets(line: &str, line_num: u32) -> Result<Option<CompletionResponse>, ResponseError> {
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
            "/create ",
            "/create ${0}",
            "AI chat to create new content",
            "AI Chat: Create",
            "Start an AI chat to create new content, e.g.
            
```smd
/create code to summarize data

/create list of highest mountains
```          
",
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
            "/edit ",
            "/edit ${0}",
            "AI chat to edit exiting content",
            "AI Chat: Edit",
            "Start an AI chat to edit existing content, e.g.
            
```smd
/edit more concise

An existing paragraph.
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

An existing paragraph.
```
",
        ),
        (
            "/fix ",
            "/fix ${0}",
            "AI chat to fix code or math",
            "AI Chat: Fix",
            "Start an AI command to fix code or math that has an error, e.g.
            
````smd
/fix

```python exec
err!
```
````
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
            "/describe ",
            "/describe ${0}",
            "AI chat to describe other content",
            "AI Chat: Describe",
            "Start an AI chat to describe other content, e.g.
            
```smd
/describe next table
```
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
            "::: prompt ",
            "::: prompt ",
            "Preview an AI prompt",
            "Prompt Preview",
            "Mainly for prompt authors to preview when a prompt is selected (based on keywords and query) and how it is rendered.

```smd
::: prompt plot of data

::: prompt create figure svg

::: prompt @create/figure-svg

::: prompt edit above table
```
",
        ),
        (
            "::: for ",
            "::: for ${1:x} in ${2:expr}\n\n$0\n\n:::",
            "Repeat a block of content",
            "For Block",
            "Content will be repeated for each value of variable in the expression, e.g.

```smd
::: for x in expr

Repeated for each value of
`x` in `expr`.

:::
```
",
        ),
        (
            "::: if ",
            "::: if ${1:expr}\n\n$0\n\n:::",
            "Activate content conditionally",
            "If Block",
            "Content will only be shown (and executed) if the expression evaluates to a truthy value,

```smd
::: if expr

Activated if `expr` is true.

:::
```
",
        ),
        (
            "::: if else",
            "::: if ${1:expr}\n\n$2\n\n::: else\n\n$0\n\n:::",
            "Activate content with fallback",
            "If Else Block",
            "Content will only be shown (and executed) if the expression evaluates to a truthy value, otherwise the fallback content will be shown,

```smd
::: if expr

Activated if `expr` is true.

::: else

Activated if `expr` is false.

:::
```
",
        ),
        (
            "::: if elif else",
            "::: if ${1:expr}\n\n$2\n\n::: else\n\n$0\n\n:::",
            "Activate content alternatives",
            "If Elif Else Block",
            "Content will only be shown (and executed) in each clause if the expression is truthy,

```smd
::: if expr1

Activated if `expr1` is true.

::: elif expr2

Activated if `expr1` is false
and `expr2` is true.

::: else

Activated if all of the above
are false.

:::
```
",
        ),
        (
            "::: include ",
            "::: include ${1:source}",
            "Include content from another document",
            "Include",
            "Insert content from another file, e.g.

```smd
::: include some/other/file.md
```
"
        ),
    ];

    let items = ITEMS
        .iter()
        .filter(|(prefix, ..)| line.is_empty() || prefix.starts_with(line))
        .enumerate()
        .map(|(index, &(prefix, body, desc, heading, docs))| {
            CompletionItem {
                label: prefix.into(),
                label_details: Some(CompletionItemLabelDetails {
                    description: Some(desc.into()),
                    ..Default::default()
                }),
                detail: Some(heading.into()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs.into(),
                })),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    // Replace the whole line with the snippet
                    range: Range {
                        start: Position {
                            line: line_num,
                            character: 0,
                        },
                        end: Position {
                            line: line_num,
                            character: u32::MAX,
                        },
                    },
                    new_text: body.into(),
                })),
                sort_text: Some(format!("{index:03}")),
                ..Default::default()
            }
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}
