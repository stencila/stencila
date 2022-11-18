use std::collections::HashMap;

use parser::{
    common::{
        eyre::{bail, Result},
        itertools::Itertools,
    },
    graph_triples::stencila_schema::*,
};

use crate::TreesitterParser;

pub trait TreesitterDecoder {
    /// Decode a `Node` from a string
    ///
    /// Similar function signature to the `Codec::from_str` function.
    fn from_str(_str: &str) -> Result<Node> {
        bail!("The `from_str` method is not implemented by this parser")
    }

    /// Get the programming language to be used in decoded `CodeChunk`s and `CodeExpression`s
    fn programming_language() -> String;

    // Is the Treesitter node an assignment of a Stencila node, and if so return the name
    // assigned and the node type
    fn inline_assignment_maybe(
        node: tree_sitter::Node,
        source: &[u8],
    ) -> Option<(String, InlineContent)> {
        if matches!(
            node.kind(),
            "equals_assignment" | "left_assignment" | "super_assignment"
        ) {
            if let Some(name) = node
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
            {
                if let Some(_func) = node
                    .child_by_field_name("value")
                    .and_then(|value| Self::is_node_call(value, source))
                {
                    return Some((
                        name.to_string(),
                        InlineContent::Parameter(Parameter {
                            name: name.to_string(),
                            ..Default::default()
                        }),
                    ));
                }
            }
        }

        None
    }

    fn assigned_inline_maybe(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> Option<InlineContent> {
        if let Some(Ok(name)) = matches!(node.kind(), "identifier").then(|| node.utf8_text(source))
        {
            if let Some(node) = assigned_inlines.get(name) {
                return Some(node.clone());
            }
        }

        None
    }

    /// Is the Treesitter node a call of a Stencila node creation function e.g. `div`, `p`, `span`,
    /// and if so return its name
    fn is_node_call(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        if !matches!(
            node.kind(),
            "call" | // py, r
            "call_expression" // js
        ) {
            return None;
        }

        let function = node
            .child_by_field_name("function")
            .and_then(|function| function.utf8_text(source).ok())
            .unwrap_or_default();

        matches!(
            function,
            // Block content
            "div" |
                "p" | 
                // Inline content
                "button" |
                "emph" |
                "expr" |
                "math" |
                "param" |
                "quote" |
                "span" |
                "strike" |
                "strong" |
                "sub" |
                "sup" |
                "under"
        )
        .then(|| function.to_string())
    }

    /// Decode some source code into an `Article`
    ///
    /// This function is intended to be used by the `ScriptCodec`. It parses the code and
    /// then iterates over the Tree-sitter tree constructing `For` and `If` blocks and putting
    /// everything else into interleaving `CodeChunk` nodes.
    fn decode(parser: &TreesitterParser, source: &str) -> Result<Node> {
        let source = source.as_bytes();

        let tree = parser.parse(source);
        let root = tree.root_node();

        let mut assigned_inlines = HashMap::new();
        let blocks = Self::decode_blocks(root, source, &mut assigned_inlines);

        Ok(Node::Article(Article {
            content: Some(blocks),
            ..Default::default()
        }))
    }

    /// Decode a block of source code into a vector of `BlockContent` nodes
    fn decode_blocks(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> Vec<BlockContent> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return Vec::new();
        }

        let ignore_first_and_last = matches!(node.kind(), "brace_list" | "statement_block");

        let mut blocks = vec![];
        let mut code = String::new();
        let mut comments = String::new();

        // Pop the code into a `CodeChunk` and push it to blocks
        macro_rules! pop_code {
            () => {
                if !code.is_empty() {
                    let trimmed = code.trim();
                    if !trimmed.is_empty() {
                        blocks.push(Self::decode_code_chunk(&trimmed));
                    }

                    code.clear();
                }
            };
        }

        // Pop the comments into blocks
        macro_rules! pop_comment {
            () => {
                if !comments.is_empty() {
                    let comment = BlockContent::Paragraph(Paragraph {
                        content: vec![InlineContent::String(comments.clone())],
                        ..Default::default()
                    });
                    blocks.push(comment);

                    comments.clear();
                }
            };
        }

        let mut index: i32 = -1;
        let mut last_comment_line = 0;
        let child_count = node.child_count() as i32;
        loop {
            index += 1;

            if index > 0 && !cursor.goto_next_sibling() {
                break;
            }

            let child = cursor.node();
            let kind = child.kind();

            if matches!(
                kind,
                "program"   // js,r,bash
                 | "module" // py
            ) {
                return Self::decode_blocks(child, source, assigned_inlines);
            }

            if matches!(
                kind,
                "for"                // r
                | "for_in_statement" // js
                | "for_statement" // py,bash
            ) {
                pop_code!();
                pop_comment!();

                blocks.push(Self::decode_for(child, source, assigned_inlines));
                continue;
            }

            if matches!(
                kind,
                "if"             // r(condition:,consequence:,alternative:)
                | "if_statement" // js,py(condition:,consequence:,alternative:) bash(condition:,command,elif_clause,else_clause)
            ) {
                pop_code!();
                pop_comment!();

                blocks.push(Self::decode_if(child, source, assigned_inlines));
                continue;
            }

            if matches!(
                kind,
                "expression_statement" // js,py
            ) {
                if let Some(grandchild) = child.named_child(0) {
                    if let Some(func) = Self::is_node_call(grandchild, source) {
                        pop_code!();
                        pop_comment!();

                        blocks.push(Self::decode_block_call(
                            &func,
                            grandchild,
                            source,
                            assigned_inlines,
                        ));
                        continue;
                    }
                }
            }

            if let Some((name, node)) = Self::inline_assignment_maybe(child, source) {
                // Register the node so that is can recognized as a node if it
                // is used as an argument in a node call
                assigned_inlines.insert(name, node);
            }

            if let Some(func) = Self::is_node_call(child, source) {
                pop_code!();
                pop_comment!();

                blocks.push(Self::decode_block_call(
                    &func,
                    child,
                    source,
                    assigned_inlines,
                ));
                continue;
            }

            if matches!(kind, "comment") {
                pop_code!();

                let current_line = child.end_position().row;
                if !comments.is_empty() {
                    for _line in last_comment_line..(current_line - 1) {
                        comments.push('\n');
                    }
                }
                last_comment_line = current_line;

                comments += child.utf8_text(source).unwrap_or_default();
                if !comments.ends_with('\n') {
                    comments.push('\n');
                }
                continue;
            }

            // Node not handled above so add to code

            pop_comment!();

            if !(ignore_first_and_last && (index == 0 || index == child_count - 1)) {
                code += child.utf8_text(source).unwrap_or_default();
                if !code.ends_with('\n') {
                    code.push('\n');
                }
            }
        }

        pop_code!();

        blocks
    }

    fn decode_code_chunk(code: &str) -> BlockContent {
        BlockContent::CodeChunk(CodeChunk {
            programming_language: Self::programming_language(),
            code: code.to_string(),
            ..Default::default()
        })
    }

    /// Decode a `for` node into a `For` node
    ///
    /// The named children to extract are:
    ///
    /// - `js`      left:      right:    body:
    /// - `py`      left:      right:    body:
    /// - `r`       name:      vector:   body:
    /// - `bash`    variable:  value:    body:
    fn decode_for(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> BlockContent {
        let symbol = node
            .child_by_field_name("left")
            .or_else(|| node.child_by_field_name("name"))
            .or_else(|| node.child_by_field_name("variable"))
            .and_then(|node| node.utf8_text(source).ok())
            .unwrap_or_default()
            .to_string();

        let code = node
            .child_by_field_name("right")
            .or_else(|| node.child_by_field_name("vector"))
            .or_else(|| node.child_by_field_name("value"))
            .and_then(|node| node.utf8_text(source).ok())
            .unwrap_or_default()
            .to_string();

        let content = node
            .child_by_field_name("body")
            .map(|block| Self::decode_blocks(block, source, assigned_inlines))
            .unwrap_or_default();

        BlockContent::For(For {
            symbol,
            code,
            content,
            ..Default::default()
        })
    }

    /// Decode a `if` node into a `If` node
    ///
    /// The named / unnamed children to extract are:
    ///
    /// - `r`         condition:   consequence:  alternative:
    /// - `js` `py`   condition:   consequence:  alternative:
    /// - `bash`      condition:   command       elif_clause  else_clause
    fn decode_if(
        _node: tree_sitter::Node,
        _source: &[u8],
        _assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> BlockContent {
        BlockContent::If(If::default())
    }

    /// Decode a call to one of the block content functions
    fn decode_block_call(
        func: &str,
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> BlockContent {
        match func {
            "div" => Self::decode_division(node, source, assigned_inlines),
            "p" => Self::decode_paragraph(node, source, assigned_inlines),
            _ => BlockContent::Paragraph(Paragraph::default()),
        }
    }

    /// Decode a call to one of the inline content functions
    fn decode_inline_call(
        func: &str,
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> InlineContent {
        Self::decode_mark(func, node, source, assigned_inlines)
    }

    fn decode_division(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> BlockContent {
        let args = node.child_by_field_name("arguments");

        let (programming_language, code) = args
            .and_then(|args| args.named_child(0))
            .map(|first| {
                // If a string with no named children (could have an "interpolation" node in Python)
                // then assume to be a Tailwind expression
                if first.kind() == "string" && first.named_child_count() == 0 {
                    return ("tailwind".to_string(), Self::unquote_string(first, source));
                }
                // Otherwise, assume an expression in the host language
                (Self::programming_language(), Self::to_string(first, source))
            })
            .unwrap_or_default();

        let content = args
            .map(|args| {
                let mut cursor = args.walk();
                let mut args = args.named_children(&mut cursor);
                args.next(); // Skip first, used above
                args.map(|arg| {
                    if Self::is_plain_string(arg) {
                        BlockContent::Paragraph(Paragraph {
                            content: vec![Self::decode_string(arg, source)],
                            ..Default::default()
                        })
                    } else if let Some(func) = Self::is_node_call(arg, source) {
                        Self::decode_block_call(&func, arg, source, assigned_inlines)
                    } else {
                        // Convert all other arguments to a code chunk
                        Self::decode_code_chunk(&Self::to_string(arg, source))
                    }
                })
                .collect_vec()
            })
            .unwrap_or_default();

        BlockContent::Division(Division {
            programming_language,
            code,
            content,
            ..Division::default()
        })
    }

    fn decode_paragraph(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> BlockContent {
        let args = node.child_by_field_name("arguments");

        let content = args
            .map(|args| Self::decode_inlines(args, source, assigned_inlines))
            .unwrap_or_default();

        BlockContent::Paragraph(Paragraph {
            content,
            ..Paragraph::default()
        })
    }

    /// Decode a Treesitter node into one of the `InlineContent` mark node types e.g. `Strong`
    fn decode_mark(
        func: &str,
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> InlineContent {
        let args = node.child_by_field_name("arguments");

        let content = args
            .map(|args| Self::decode_inlines(args, source, assigned_inlines))
            .unwrap_or_default();

        match func {
            "emph" => InlineContent::Emphasis(Emphasis {
                content,
                ..Default::default()
            }),
            "quote" => InlineContent::Quote(Quote {
                content,
                ..Default::default()
            }),
            "strike" => InlineContent::Strikeout(Strikeout {
                content,
                ..Default::default()
            }),
            "strong" => InlineContent::Strong(Strong {
                content,
                ..Default::default()
            }),
            "sub" => InlineContent::Subscript(Subscript {
                content,
                ..Default::default()
            }),
            "sup" => InlineContent::Superscript(Superscript {
                content,
                ..Default::default()
            }),
            "under" => InlineContent::Underline(Underline {
                content,
                ..Default::default()
            }),
            // This should never get reached, but in case it does...
            _ => InlineContent::String(Self::unquote_string(node, source)),
        }
    }

    fn decode_inlines(
        node: tree_sitter::Node,
        source: &[u8],
        assigned_inlines: &mut HashMap<String, InlineContent>,
    ) -> Vec<InlineContent> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor)
            .map(|child| {
                if Self::is_plain_string(child) {
                    Self::decode_string(child, source)
                } else if let Some(node) =
                    Self::assigned_inline_maybe(child, source, assigned_inlines)
                {
                    node
                } else if let Some(func) = Self::is_node_call(child, source) {
                    Self::decode_inline_call(&func, child, source, assigned_inlines)
                } else {
                    InlineContent::CodeExpression(CodeExpression {
                        programming_language: Self::programming_language(),
                        code: child.utf8_text(source).unwrap_or_default().to_string(),
                        ..Default::default()
                    })
                }
            })
            .collect_vec()
    }

    /// Decode a Treesitter node into an `InlineContent::String`
    fn decode_string(node: tree_sitter::Node, source: &[u8]) -> InlineContent {
        InlineContent::String(Self::unquote_string(node, source))
    }

    /// Determine is a node is a plain string (not a template string / interpolated string)
    fn is_plain_string(node: tree_sitter::Node) -> bool {
        node.kind() == "string" && node.named_child_count() == 0
    }

    /// Remove the surrounding quotes (single or double) from a Treesitter node that is a string
    fn unquote_string(node: tree_sitter::Node, source: &[u8]) -> String {
        let mut chars = node.utf8_text(source).unwrap_or_default().chars();
        chars.next();
        chars.next_back();
        chars.as_str().to_string()
    }

    /// Get the source string of a Treesitter node
    fn to_string(node: tree_sitter::Node, source: &[u8]) -> String {
        node.utf8_text(source).unwrap_or_default().to_string()
    }
}
