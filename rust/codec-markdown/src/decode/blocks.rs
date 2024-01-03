//! Parsing of Stencila custom Markdown extensions for `Block` nodes

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{alpha1, char, multispace0, multispace1, none_of},
    combinator::{all_consuming, map, opt, recognize},
    multi::{many_m_n, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use codec::{
    common::itertools::Itertools,
    schema::{
        Admonition, Block, CallArgument, CallBlock, Claim, Cord, Figure, ForBlock, Form,
        FormDeriveAction, FormOptions, IfBlockClause, IncludeBlock, Inline, IntegerOrString,
        MathBlock, Node, Section, StyledBlock, Text,
    },
};

use super::parse::{assignee, curly_attrs, node_to_from_str, node_to_string, symbol};

// Note: Most of these parsers are all consuming because they are used
// to test a match against a whole line.

/// Detect at least three semicolons
fn semis(input: &str) -> IResult<&str, &str> {
    recognize(many_m_n(3, 100, char(':')))(input)
}

/// Parse an [`Admonition`] from blocks in a block quote
pub fn admonition(content: &mut Vec<Block>) -> Option<Admonition> {
    if let Some(Block::Paragraph(para)) = content.first_mut() {
        if let Some(Inline::Text(Text { value: text, .. })) = para.content.first_mut() {
            if text.starts_with("[!") {
                if let Some(mut finish) = text.find(']') {
                    let admonition_type = text[2..finish].parse().unwrap_or_default();

                    let is_folded = match text.chars().nth(finish + 1) {
                        Some('+') => {
                            finish += 1;
                            Some(true)
                        }
                        Some('-') => {
                            finish += 1;
                            Some(false)
                        }
                        _ => None,
                    };

                    // Remove the prefix from the para and then make any remaining
                    // content the title
                    *text = Cord::from(text[finish + 1..].trim_start());
                    let title = if text.is_empty() {
                        para.content.drain(1..)
                    } else {
                        para.content.drain(0..)
                    }
                    .collect_vec();
                    let title = if title.is_empty() { None } else { Some(title) };

                    let content = content.drain(1..).collect();

                    return Some(Admonition {
                        admonition_type,
                        title,
                        is_folded,
                        content,
                        ..Default::default()
                    });
                }
            }
        }
    }

    None
}

/// Parse a [`MathBlock`] node
pub fn math_block(input: &str) -> IResult<&str, MathBlock> {
    map(
        all_consuming(delimited(tag("$$"), is_not("$"), tag("$$"))),
        |code: &str| {
            // Remove leading and trailing spaces (not newlines because pulldown_cmark
            // converts newlines in paragraphs into spaces)
            let mut code = Cord::from(code);
            if code.starts_with(' ') {
                code.remove(0);
            }
            if code.ends_with(' ') {
                code.pop();
            }

            MathBlock {
                code,
                math_language: Some(String::from("tex")),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse an [`Include`] node
pub fn include(input: &str) -> IResult<&str, IncludeBlock> {
    map(
        all_consuming(preceded(
            char('/'),
            // Exclude '(' from source to avoid clash with a `Call`
            tuple((is_not("({"), opt(curly_attrs))),
        )),
        |(source, options)| {
            let mut options: HashMap<String, _> = options.unwrap_or_default().into_iter().collect();

            IncludeBlock {
                source: source.to_string(),
                media_type: options.remove("format").flatten().map(node_to_string),
                select: options.remove("select").flatten().map(node_to_string),
                auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse a [`Call`] node
pub fn call(input: &str) -> IResult<&str, CallBlock> {
    map(
        all_consuming(preceded(
            char('/'),
            tuple((
                is_not("("),
                delimited(
                    char('('),
                    separated_list0(
                        alt((delimited(multispace0, tag(","), multispace0), multispace1)),
                        call_arg,
                    ),
                    char(')'),
                ),
                opt(curly_attrs),
            )),
        )),
        |(source, args, options)| {
            let mut options: HashMap<String, _> = options.unwrap_or_default().into_iter().collect();

            CallBlock {
                source: source.to_string(),
                arguments: args,
                media_type: options.remove("format").flatten().map(node_to_string),
                select: options.remove("select").flatten().map(node_to_string),
                auto_exec: options.remove("auto").flatten().and_then(node_to_from_str),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse an argument inside a set of curly braced arguments.
///
/// Arguments must be key-value or key-symbol pairs separated by `=`.
fn call_arg(input: &str) -> IResult<&str, CallArgument> {
    map(
        // TODO allow for programming language to be specified
        pair(
            terminated(symbol, delimited(multispace0, tag("="), multispace0)),
            alt((delimited(char('`'), is_not("`"), char('`')), is_not(", )"))),
        ),
        |(name, code)| CallArgument {
            name,
            code: code.into(),
            ..Default::default()
        },
    )(input)
}

/// Start an [`InstructBlock`]
pub fn instruct_block_start(input: &str) -> IResult<&str, (Option<&str>, &str, bool)> {
    let (input, has_content) = if let Some(stripped) = input.strip_suffix("%>") {
        (stripped, true)
    } else {
        (input, false)
    };

    let (remains, (assignee, text)) = all_consuming(preceded(
        pair(tag("%%"), multispace0),
        pair(
            opt(delimited(char('@'), assignee, multispace1)),
            is_not("\n"),
        ),
    ))(input)?;

    Ok((remains, (assignee, text.trim(), has_content)))
}

/// End an [`InstructBlock`] with content
pub fn instruct_block_end(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("%%"))(input)
}

/// Parse the start or end an [`InsertBlock`] node
pub fn insert_block(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("++"))(input)
}

/// Parse the start or end of a [`DeleteBlock`] node
pub fn delete_block(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("--"))(input)
}

/// Parse the start or end of a [`ReplaceBlock`] node
pub fn replace_block(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("~~"))(input)
}

/// Parse the separator of a [`ReplaceBlock`] node
pub fn replace_block_separator(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("~>"))(input)
}

/// Parse the start or end of a [`ModifyBlock`] node
pub fn modify_block(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("!!"))(input)
}

/// Parse the separator of a [`ModifyBlock`] node
pub fn modify_block_separator(input: &str) -> IResult<&str, &str> {
    all_consuming(tag("!>"))(input)
}

/// Parse a [`Section`] node
pub fn section(input: &str) -> IResult<&str, Section> {
    map(
        all_consuming(preceded(tuple((semis, multispace0)), alpha1)),
        |typ| Section {
            section_type: typ.parse().ok(),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Figure`] node
pub fn figure(input: &str) -> IResult<&str, Figure> {
    map(
        all_consuming(preceded(
            tuple((
                semis,
                multispace0,
                alt((tag("figure"), tag("fig"))),
                multispace0,
            )),
            opt(is_not("\r\n")),
        )),
        |label| Figure {
            label: label.map(|label| label.to_string()),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Claim`] node
pub fn claim(input: &str) -> IResult<&str, Claim> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            tuple((
                alt((
                    tag("corollary"),
                    tag("hypothesis"),
                    tag("lemma"),
                    tag("postulate"),
                    tag("proof"),
                    tag("proposition"),
                    tag("statement"),
                    tag("theorem"),
                )),
                opt(preceded(multispace1, is_not("\r\n"))),
            )),
        )),
        |(claim_type, label)| Claim {
            claim_type: claim_type.parse().unwrap_or_default(),
            label: label.map(String::from),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`ForBlock`] node
pub fn for_block(input: &str) -> IResult<&str, ForBlock> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("for"), multispace1)),
            tuple((
                separated_pair(
                    symbol,
                    tuple((multispace1, tag("in"), multispace1)),
                    is_not("{"),
                ),
                opt(preceded(
                    multispace0,
                    delimited(char('{'), is_not("}"), char('}')),
                )),
            )),
        )),
        |((symbol, expr), lang)| ForBlock {
            symbol,
            code: Cord::new(expr.trim()),
            programming_language: lang.map(|lang| lang.trim().to_string()),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Form`] node
pub fn form(input: &str) -> IResult<&str, Form> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0, tag("form"), multispace0)),
            opt(curly_attrs),
        )),
        |options| {
            let mut options: HashMap<_, _> = options.unwrap_or_default().into_iter().collect();

            let derive_from = options.remove("from").flatten().map(node_to_string);

            let derive_action = options.remove("action").flatten().and_then(|node| {
                match node_to_string(node).to_lowercase().as_str() {
                    "create" => Some(FormDeriveAction::Create),
                    "update" => Some(FormDeriveAction::Update),
                    "delete" => Some(FormDeriveAction::Delete),
                    _ => None,
                }
            });

            let derive_item = options
                .remove("item")
                .flatten()
                .and_then(|node| match node {
                    Node::Integer(int) => Some(IntegerOrString::Integer(int)),
                    Node::String(string) => Some(IntegerOrString::String(string)),
                    _ => None,
                });

            Form {
                options: Box::new(FormOptions {
                    derive_from,
                    derive_action,
                    derive_item,
                    ..Default::default()
                }),
                ..Default::default()
            }
        },
    )(input)
}

/// Parse an `if` or `elif` section into an [`IfBlockClause`]
pub fn if_elif(input: &str) -> IResult<&str, (bool, IfBlockClause)> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            tuple((
                alt((tag("if"), tag("elif"))),
                alt((
                    preceded(
                        multispace1,
                        delimited(char('`'), escaped(none_of("`"), '\\', char('`')), char('`')),
                    ),
                    preceded(multispace1, is_not("{")),
                    multispace0,
                )),
                opt(curly_attrs),
            )),
        )),
        |(tag, expr, options)| {
            (
                tag == "if",
                IfBlockClause {
                    code: Cord::from(expr.trim()),
                    programming_language: options
                        .iter()
                        .flatten()
                        .next()
                        .map(|tuple| tuple.0.trim().to_string()),
                    ..Default::default()
                },
            )
        },
    )(input)
}

/// Parse an `else` section
pub fn else_block(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((
        semis,
        multispace0,
        tag("else"),
        // Allow for, but ignore, trailing content
        opt(pair(multispace1, is_not(""))),
    ))))(input)
}

/// Parse a [`StyledBlock`] node
pub fn styled_block(input: &str) -> IResult<&str, StyledBlock> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            tuple((
                opt(terminated(alpha1, multispace0)),
                delimited(char('{'), is_not("}"), char('}')),
            )),
        )),
        |(lang, code)| StyledBlock {
            code: Cord::from(code),
            style_language: lang.map(|lang| lang.into()),
            ..Default::default()
        },
    )(input)
}

/// Parse a separator in a division
pub fn sep(input: &str) -> IResult<&str, &str> {
    recognize(pair(many_m_n(2, 99, char(':')), char('>')))(input)
}

/// Parse the end of a division
pub fn end(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((semis, multispace0))))(input)
}

#[cfg(test)]
mod tests {
    use codec::schema::ClaimType;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calls() {
        assert_eq!(
            call("/file.md()").unwrap().1,
            CallBlock {
                source: "file.md".to_string(),
                ..Default::default()
            }
        );
        assert_eq!(
            call("/file.md(a=1)").unwrap().1,
            CallBlock {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "a".to_string(),
                    code: "1".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
        assert_eq!(
            call(r#"/file.md(parAm_eter_1="string")"#).unwrap().1,
            CallBlock {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "parAm_eter_1".to_string(),
                    code: "\"string\"".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
        assert_eq!(
            call("/file.md(a=1.23 b=symbol c='string')").unwrap().1,
            CallBlock {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        code: "1.23".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        code: "symbol".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        code: "'string'".into(),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
        );
        assert_eq!(
            call("/file.md(a=1,b = 2  , c=3, d =4)").unwrap().1,
            CallBlock {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        code: "1".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        code: "2".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        code: "3".into(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "d".to_string(),
                        code: "4".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_claim() {
        assert_eq!(
            claim("::: hypothesis").unwrap().1,
            Claim {
                claim_type: ClaimType::Hypothesis,
                ..Default::default()
            }
        );

        assert_eq!(
            claim("::: lemma Lemma 1").unwrap().1,
            Claim {
                claim_type: ClaimType::Lemma,
                label: Some(String::from("Lemma 1")),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_for() {
        // Simple
        assert_eq!(
            for_block("::: for item in expr").unwrap().1,
            ForBlock {
                symbol: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            for_block(":::for item  in    expr").unwrap().1,
            ForBlock {
                symbol: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            for_block("::: for item in expr {python}").unwrap().1,
            ForBlock {
                symbol: "item".to_string(),
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            for_block("::: for i in 1:10").unwrap().1,
            ForBlock {
                symbol: "i".to_string(),
                code: "1:10".into(),
                ..Default::default()
            }
        );
        assert_eq!(
            for_block("::: for row in select * from table { sql }")
                .unwrap()
                .1,
            ForBlock {
                symbol: "row".to_string(),
                code: "select * from table".into(),
                programming_language: Some("sql".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_form() {
        assert_eq!(form("::: form").unwrap().1, Form::default());
    }

    #[test]
    fn test_if() {
        // Simple
        assert_eq!(
            if_elif("::: if expr").unwrap().1 .1,
            IfBlockClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            if_elif(":::if    expr").unwrap().1 .1,
            IfBlockClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            if_elif("::: if expr {python}").unwrap().1 .1,
            IfBlockClause {
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            if_elif("::: if a > 1 and b[8] < 1.23").unwrap().1 .1,
            IfBlockClause {
                code: "a > 1 and b[8] < 1.23".into(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_end() {
        assert!(end(":::").is_ok());
        assert!(end("::::").is_ok());
        assert!(end("::::::").is_ok());

        assert!(end(":::some chars").is_err());
        assert!(end("::").is_err());
        assert!(end(":").is_err());
    }
}
