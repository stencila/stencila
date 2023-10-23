//! Parsing of Stencila custom Markdown extensions for `Block` nodes

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, multispace0, multispace1, none_of},
    combinator::{all_consuming, map, opt, recognize},
    multi::{many_m_n, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use codec::schema::{
    Call, CallArgument, Cord, Division, For, Form, FormDeriveAction, FormOptions, IfClause,
    Include, IntegerOrString, MathBlock, Node, Section,
};

use super::parse::{curly_attrs, node_to_from_str, node_to_string, symbol};

/// Note: Most of these parsers are all consuming because they are used
/// to test a match against a whole line.

/// Detect at least three semicolons
fn semis(input: &str) -> IResult<&str, &str> {
    recognize(many_m_n(3, 100, char(':')))(input)
}

/// Parse a [`MathBlock`] node
pub fn parse_math_block(input: &str) -> IResult<&str, MathBlock> {
    map(
        all_consuming(delimited(tag("$$"), is_not("$"), tag("$$"))),
        |code: &str| MathBlock {
            code: Cord::from(code.trim()),
            math_language: "tex".to_string(),
            ..Default::default()
        },
    )(input)
}

/// Parse an [`Include`] node
pub fn parse_include(input: &str) -> IResult<&str, Include> {
    map(
        all_consuming(preceded(
            char('/'),
            // Exclude '(' from source to avoid clash with a `Call`
            tuple((is_not("({"), opt(curly_attrs))),
        )),
        |(source, options)| {
            let mut options: HashMap<String, _> = options.unwrap_or_default().into_iter().collect();

            Include {
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
pub fn parse_call(input: &str) -> IResult<&str, Call> {
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

            Call {
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
            code: code.to_string(),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Section`] node
pub fn parse_section(input: &str) -> IResult<&str, Section> {
    map(all_consuming(tuple((semis, multispace0))), |_| {
        Section::default()
    })(input)
}

/// Parse a [`Division`] node
pub fn parse_division(input: &str) -> IResult<&str, Division> {
    map(
        all_consuming(preceded(
            tuple((semis, multispace0)),
            alt((
                // TODO use similar approach as for if etc of only escaping with backticks if needed
                // and guessing languages
                // TODO allow for divs with no style
                tuple((
                    delimited(char('`'), is_not("`"), char('`')),
                    delimited(char('{'), is_not("}"), char('}')),
                )),
                map(is_not("\r\n"), |text| (text, "tailwind")),
            )),
        )),
        |(code, style_language)| Division {
            code: Cord::from(code.trim()),
            style_language: Some(style_language.to_string()),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`For`] node
pub fn parse_for(input: &str) -> IResult<&str, For> {
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
        |((symbol, expr), lang)| For {
            symbol,
            code: Cord::new(expr.trim()),
            programming_language: lang.map(|lang| lang.trim().to_string()),
            ..Default::default()
        },
    )(input)
}

/// Parse a [`Form`] node
pub fn parse_form(input: &str) -> IResult<&str, Form> {
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

/// Parse an `if` or `elif` section into an [`IfClause`]
pub fn parse_if_elif(input: &str) -> IResult<&str, (bool, IfClause)> {
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
                IfClause {
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
pub fn parse_else(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((
        semis,
        multispace0,
        tag("else"),
        // Allow for, but ignore, trailing content
        opt(pair(multispace1, is_not(""))),
    ))))(input)
}

/// Parse the end of a division
pub fn parse_end(input: &str) -> IResult<&str, &str> {
    all_consuming(recognize(tuple((semis, multispace0))))(input)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_calls() {
        assert_eq!(
            parse_call("/file.md()").unwrap().1,
            Call {
                source: "file.md".to_string(),
                ..Default::default()
            }
        );
        assert_eq!(
            parse_call("/file.md(a=1)").unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "a".to_string(),
                    code: "1".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
        assert_eq!(
            parse_call(r#"/file.md(parAm_eter_1="string")"#).unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: vec![CallArgument {
                    name: "parAm_eter_1".to_string(),
                    code: "\"string\"".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
        assert_eq!(
            parse_call("/file.md(a=1.23 b=symbol c='string')")
                .unwrap()
                .1,
            Call {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        code: "1.23".to_string(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        code: "symbol".to_string(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        code: "'string'".to_string(),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
        );
        assert_eq!(
            parse_call("/file.md(a=1,b = 2  , c=3, d =4)").unwrap().1,
            Call {
                source: "file.md".to_string(),
                arguments: vec![
                    CallArgument {
                        name: "a".to_string(),
                        code: "1".to_string(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "b".to_string(),
                        code: "2".to_string(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "c".to_string(),
                        code: "3".to_string(),
                        ..Default::default()
                    },
                    CallArgument {
                        name: "d".to_string(),
                        code: "4".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_for() {
        // Simple
        assert_eq!(
            parse_for("::: for item in expr").unwrap().1,
            For {
                symbol: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            parse_for(":::for item  in    expr").unwrap().1,
            For {
                symbol: "item".to_string(),
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            parse_for("::: for item in expr {python}").unwrap().1,
            For {
                symbol: "item".to_string(),
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            parse_for("::: for i in 1:10").unwrap().1,
            For {
                symbol: "i".to_string(),
                code: "1:10".into(),
                ..Default::default()
            }
        );
        assert_eq!(
            parse_for("::: for row in select * from table { sql }")
                .unwrap()
                .1,
            For {
                symbol: "row".to_string(),
                code: "select * from table".into(),
                programming_language: Some("sql".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_form() {
        assert_eq!(parse_form("::: form").unwrap().1, Form::default());
    }

    #[test]
    fn test_if() {
        // Simple
        assert_eq!(
            parse_if_elif("::: if expr").unwrap().1 .1,
            IfClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With less/extra spacing
        assert_eq!(
            parse_if_elif(":::if    expr").unwrap().1 .1,
            IfClause {
                code: "expr".into(),
                ..Default::default()
            }
        );

        // With language specified
        assert_eq!(
            parse_if_elif("::: if expr {python}").unwrap().1 .1,
            IfClause {
                code: "expr".into(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }
        );

        // With more complex expression
        assert_eq!(
            parse_if_elif("::: if a > 1 and b[8] < 1.23").unwrap().1 .1,
            IfClause {
                code: "a > 1 and b[8] < 1.23".into(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_end() {
        assert!(parse_end(":::").is_ok());
        assert!(parse_end("::::").is_ok());
        assert!(parse_end("::::::").is_ok());

        assert!(parse_end(":::some chars").is_err());
        assert!(parse_end("::").is_err());
        assert!(parse_end(":").is_err());
    }
}
