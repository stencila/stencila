//! Parsing of Stencila custom Markdown extensions for `Inline` nodes

use std::collections::HashMap;

use codec_text_trait::TextCodec;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_until, take_while1},
    character::complete::{anychar, char, digit1, multispace0, multispace1},
    combinator::{map, not, opt, peek},
    error::{Error, ErrorKind},
    multi::{fold_many0, many_till, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Err, IResult,
};

use codec::{
    common::indexmap::IndexMap,
    schema::{
        shortcuts::{dei, isi, mi, rei, stk, sub, sup, t},
        BooleanValidator, Button, Cite, CiteGroup, CodeExpression, CodeInline, Cord,
        DateTimeValidator, DateValidator, DurationValidator, EnumValidator, Inline,
        InstructionInline, InstructionInlineOptions, IntegerValidator, Message, MessagePart,
        ModifyInline, Node, NumberValidator, Parameter, ParameterOptions, StringValidator,
        StyledInline, TimeValidator, TimestampValidator, Validator,
    },
};

use super::parse::{
    assignee, curly_attrs, node_to_option_date, node_to_option_datetime, node_to_option_duration,
    node_to_option_i64, node_to_option_number, node_to_option_time, node_to_option_timestamp,
    node_to_string, symbol,
};

/// Parse a string into a vector of `Inline` nodes
///
/// Whilst accumulating, will combine adjacent `Text` nodes.
/// This is necessary because of the catch all `character` parser.
pub fn inlines(input: &str) -> IResult<&str, Vec<Inline>> {
    fold_many0(
        alt((
            button,
            code_attrs,
            code_expr,
            cite_group,
            cite,
            math,
            parameter,
            styled_inline,
            strikeout,
            subscript,
            superscript,
            instruction_inline,
            insert_inline,
            delete_inline,
            replace_inline,
            modify_inline,
            string,
            character,
        )),
        Vec::new,
        |mut vec: Vec<Inline>, node| {
            if let Inline::Text(text) = &node {
                match vec.last_mut() {
                    Some(Inline::Text(last)) => last.value.push_str(&text.value),
                    _ => vec.push(node),
                }
            } else {
                vec.push(node)
            }
            vec
        },
    )(input)
}

/// Parse a string into a vector of `Inline` nodes falling back to a single `Text` nodes on error
pub fn inlines_or_text(input: &str) -> Vec<Inline> {
    inlines(input).map_or_else(|_| vec![t(input)], |(.., inlines)| inlines)
}

/// Parse inline code with attributes in curly braces
/// e.g. `\`code\`{attr1 attr2}` into a `CodeFragment`, `CodeExpression`
/// or `MathFragment` node.
///
/// The `curly_attrs` are optional because plain `CodeFragment`s also end up being
/// passed to this function
fn code_attrs(input: &str) -> IResult<&str, Inline> {
    map(
        pair(
            delimited(char('`'), take_until("`"), char('`')),
            opt(curly_attrs),
        ),
        |(code, options)| {
            let Some(options) = options else {
                return Inline::CodeInline(CodeInline {
                    code: code.into(),
                    ..Default::default()
                });
            };

            let mut lang = None;
            let mut exec = false;
            let mut auto_exec = None;

            for (name, value) in options {
                if name == "exec" {
                    exec = true
                } else if value.is_none() {
                    lang = Some(name);
                } else if name == "auto" {
                    if let Some(value) = value {
                        auto_exec = Some(value.to_text().0.parse().unwrap_or_default())
                    }
                }
            }

            if exec {
                Inline::CodeExpression(CodeExpression {
                    code: code.into(),
                    programming_language: lang,
                    auto_exec,
                    ..Default::default()
                })
            } else {
                match lang.as_deref() {
                    Some("asciimath") | Some("mathml") | Some("latex") | Some("tex") => {
                        mi(code, lang)
                    }
                    _ => Inline::CodeInline(CodeInline {
                        code: code.into(),
                        programming_language: lang,
                        ..Default::default()
                    }),
                }
            }
        },
    )(input)
}

/// Parse a [`StyledInline`].
fn styled_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((
            delimited(char('['), take_until_unbalanced('[', ']'), char(']')),
            delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
            opt(delimited(char('{'), is_not("}"), char('}'))),
        )),
        |(content, code, lang): (&str, &str, Option<&str>)| {
            Inline::StyledInline(StyledInline {
                content: inlines_or_text(content),
                code: code.into(),
                style_language: lang.map(|lang| lang.into()),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a `Parameter`.
fn parameter(input: &str) -> IResult<&str, Inline> {
    map(
        pair(
            delimited(tag("&["), opt(symbol), char(']')),
            opt(curly_attrs),
        ),
        |(name, attrs)| {
            let attrs = attrs.unwrap_or_default();
            let first = attrs
                .first()
                .map(|(name, ..)| Some(Node::String(name.clone())));
            let mut options: HashMap<String, Option<Node>> = attrs.into_iter().collect();

            let typ = options
                .remove("type")
                .or(first.clone())
                .flatten()
                .map(node_to_string);
            let typ = typ.as_deref();

            let label = options.remove("label").flatten().map(node_to_string);

            let validator = if matches!(typ, Some("boolean")) || matches!(typ, Some("bool")) {
                Some(Validator::BooleanValidator(BooleanValidator::default()))
            } else if matches!(typ, Some("enum")) {
                let values = options
                    .remove("values")
                    .or_else(|| options.remove("vals"))
                    .flatten();
                let values = match values {
                    Some(node) => match node {
                        // Usually the supplied node is an array, which we need to convert
                        // to a vector of `Node`s
                        Node::Array(array) => array
                            .iter()
                            .map(|primitive| primitive.clone().into())
                            .collect(),
                        _ => vec![node],
                    },
                    None => vec![],
                };
                Some(Validator::EnumValidator(EnumValidator {
                    values,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("integer")) || matches!(typ, Some("int")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("emin"))
                    .flatten()
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("emax"))
                    .flatten()
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("multiple_of")
                    .or_else(|| options.remove("mult"))
                    .or_else(|| options.remove("step"))
                    .flatten()
                    .and_then(node_to_option_number);
                Some(Validator::IntegerValidator(IntegerValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("number")) || matches!(typ, Some("num")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("emin"))
                    .flatten()
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("exclusive_minimum")
                    .or_else(|| options.remove("emax"))
                    .flatten()
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("multiple_of")
                    .or_else(|| options.remove("mult"))
                    .flatten()
                    .and_then(node_to_option_number);
                Some(Validator::NumberValidator(NumberValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("string")) || matches!(typ, Some("str")) {
                let min_length = options
                    .remove("min_length")
                    .or_else(|| options.remove("minlength"))
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_i64);
                let max_length = options
                    .remove("max_length")
                    .or_else(|| options.remove("maxlength"))
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_i64);
                let pattern = options
                    .remove("pattern")
                    .or_else(|| options.remove("regex"))
                    .flatten()
                    .map(node_to_string);
                Some(Validator::StringValidator(StringValidator {
                    min_length,
                    max_length,
                    pattern,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("date")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_date);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_date);
                Some(Validator::DateValidator(DateValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("time")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_time);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_time);
                Some(Validator::TimeValidator(TimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("datetime")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_datetime);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_datetime);
                Some(Validator::DateTimeValidator(DateTimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("timestamp")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_timestamp);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_timestamp);
                Some(Validator::TimestampValidator(TimestampValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("duration")) {
                let minimum = options
                    .remove("minimum")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_duration);
                let maximum = options
                    .remove("maximum")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_duration);
                Some(Validator::DurationValidator(DurationValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else {
                None
            };

            let derived_from = options
                .remove("derived-from")
                .or_else(|| options.remove("from"))
                .flatten()
                .map(node_to_string);

            let name = name
                .or_else(|| {
                    derived_from
                        .clone()
                        .map(|from| from.split('.').last().unwrap_or(from.as_str()).to_string())
                })
                .unwrap_or_else(|| "unnamed".to_string());

            let default = options
                .remove("default")
                .or_else(|| options.remove("def"))
                .flatten()
                .map(Box::new);

            let value = options
                .remove("value")
                .or_else(|| options.remove("val"))
                .flatten()
                .map(Box::new);

            Inline::Parameter(Parameter {
                name,
                value,
                options: Box::new(ParameterOptions {
                    label,
                    validator,
                    default,
                    derived_from,
                    ..Default::default()
                }),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a `Button`
fn button(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((
            delimited(tag("#["), is_not("]"), char(']')),
            opt(delimited(char('`'), is_not("`"), char('`'))),
            opt(curly_attrs),
        )),
        |(name, condition, options)| {
            let mut options: IndexMap<String, Option<Node>> =
                options.unwrap_or_default().into_iter().collect();

            let programming_language = if let Some((lang, None)) = options.first() {
                Some(lang.clone())
            } else {
                None
            };

            let code = condition.map_or_else(Cord::default, Cord::from);

            let label = options.swap_remove("label").flatten().map(node_to_string);

            Inline::Button(Button {
                name: name.to_string(),
                programming_language,
                code,
                label,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse double brace surrounded text into a `CodeExpression`.
///
/// This supports the Jupyter "Python Markdown" extension syntax for
/// interpolated variables / expressions: `{{x}}`
///
/// Does not support the single curly brace syntax (as in Python, Rust and JSX) i.e. `{x}`
/// given that is less specific and could conflict with other user content.
///
/// Does not support JavaScript style "dollared-brace" syntax i.e. `${x}` since some
/// at least some Markdown parsers seem to parse that as TeX math (even though there
/// is no closing brace).
///
/// The language of the code expression can be added in a curly brace suffix.
/// e.g. `{{2 * 2}}{r}` is equivalent to `\`r 2 * 2\``{r exec} in Markdown or to
/// `\`r 2 * 2\` in R Markdown.
fn code_expr(input: &str) -> IResult<&str, Inline> {
    map(
        pair(
            delimited(tag("{{"), take_until("}}"), tag("}}")),
            opt(delimited(char('{'), take_until("}"), char('}'))),
        ),
        |(code, options): (&str, Option<&str>)| {
            let code = code.into();
            let lang = match options {
                Some(attrs) => {
                    let attrs = attrs.split_whitespace().collect::<Vec<&str>>();
                    attrs.first().map(|item| item.to_string())
                }
                None => None,
            };
            Inline::CodeExpression(CodeExpression {
                code,
                programming_language: lang,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a narrative `Cite` node
///
/// This attempts to follow Pandoc's citation handling as closely as possible
/// (see <https://pandoc.org/MANUAL.html#citations>).
///
/// The following properties of a `Cite` are parsed:
///   - [x] target
///   - [ ] citation_mode
///   - [ ] page_start
///   - [ ] page_end
///   - [ ] pagination
///   - [ ] citation_prefix
///   - [ ] citation_suffix
///   - [ ] citation_intent
fn cite(input: &str) -> IResult<&str, Inline> {
    // TODO: Parse more properties of citations
    map(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| {
            let target = res.into();
            Inline::Cite(Cite {
                target,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a `CiteGroup` node or parenthetical `Cite` node.
///
/// If there is only one citation within square brackets then a parenthetical `Cite` node is
/// returned. Otherwise, the `Cite` nodes are grouped into into a `CiteGroup`.
fn cite_group(input: &str) -> IResult<&str, Inline> {
    let cite = map(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| {
            let target = res.into();
            Inline::Cite(Cite {
                target,
                ..Default::default()
            })
        },
    );

    map(
        delimited(
            char('['),
            separated_list1(tuple((multispace0, tag(";"), multispace0)), cite),
            char(']'),
        ),
        |items: Vec<Inline>| {
            if items.len() == 1 {
                items[0].clone()
            } else {
                Inline::CiteGroup(CiteGroup {
                    items: items
                        .iter()
                        .filter_map(|item| match item {
                            Inline::Cite(cite) => Some(cite),
                            _ => None,
                        })
                        .cloned()
                        .collect(),
                    ..Default::default()
                })
            }
        },
    )(input)
}

/// Parse a string into an `Inline` node
///
/// This attempts to follow Pandoc's math parsing as closely as possible
/// (see <https://pandoc.org/MANUAL.html#math>).
fn math(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            // Pandoc: "opening $ must have a non-space character immediately to its right"
            tuple((char('$'), peek(not(multispace1)))),
            take_until("$"),
            // Pandoc: "the closing $ must have a non-space character immediately to its left,
            // and must not be followed immediately by a digit"
            tuple((peek(not(multispace1)), char('$'), peek(not(digit1)))),
        ),
        |code: &str| mi(code, Some(String::from("tex"))),
    )(input)
}

/// Parse a string into a `Strikeout` node
fn strikeout(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("~~"), take_until("~~"), tag("~~")),
        |content: &str| stk(inlines_or_text(content)),
    )(input)
}

/// Parse a string into a `Subscript` node
fn subscript(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            // Only match single tilde, because doubles are for `Strikeout`
            tuple((char('~'), peek(not(char('~'))))),
            take_until("~"),
            char('~'),
        ),
        |content: &str| sub(inlines_or_text(content)),
    )(input)
}

/// Parse a string into a `Superscript` node
fn superscript(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(char('^'), take_until("^"), char('^')),
        |content: &str| sup(inlines_or_text(content)),
    )(input)
}

/// Parse a string into a `InstructionInline` node
fn instruction_inline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            terminated(tag("{%%"), multispace0),
            tuple((
                opt(delimited(char('@'), assignee, multispace1)),
                map(
                    many_till(anychar, peek(alt((tag("%>"), tag("%%}"))))),
                    |(chars, ..)| -> String { chars.iter().collect() },
                ),
                opt(preceded(tag("%>"), take_until("%%}"))),
            )),
            tag("%%}"),
        ),
        |(assignee, text, content)| {
            Inline::InstructionInline(InstructionInline {
                messages: vec![Message {
                    parts: vec![MessagePart::String(text.trim().to_string())],
                    ..Default::default()
                }],
                options: Box::new(InstructionInlineOptions {
                    assignee: assignee.map(|handle| handle.to_string()),
                    ..Default::default()
                }),
                content: content.map(inlines_or_text),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a `InsertInline` node
fn insert_inline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("{++"), take_until("++}"), tag("++}")),
        |content: &str| isi(inlines_or_text(content)),
    )(input)
}

/// Parse a string into a `DeleteInline` node
fn delete_inline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("{--"), take_until("--}"), tag("--}")),
        |content: &str| dei(inlines_or_text(content)),
    )(input)
}

/// Parse a string into a `ReplaceInline` node
fn replace_inline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            tag("{~~"),
            pair(terminated(take_until("~>"), tag("~>")), take_until("~~}")),
            tag("~~}"),
        ),
        |(content, replacement)| rei(inlines_or_text(content), inlines_or_text(replacement)),
    )(input)
}

/// Parse a string into a `ModifyInline` node
///
/// Note that the parsed content and modification preview are ignored
/// since this is "read-only".
fn modify_inline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            tag("{!!"),
            pair(terminated(take_until("!>"), tag("!>")), take_until("!!}")),
            tag("!!}"),
        ),
        |(_content, _preview)| Inline::ModifyInline(ModifyInline::default()),
    )(input)
}

/// Accumulate characters into a `String` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &str) -> IResult<&str, Inline> {
    const CHARS: &str = "~@#$^&[{`";
    map(take_while1(|chr: char| !CHARS.contains(chr)), t)(input)
}

/// Take a single character into a `String` node
///
/// Necessary so that the characters no consumed by `string` are not lost.
fn character(input: &str) -> IResult<&str, Inline> {
    map(take(1usize), t)(input)
}

/// Take characters until `opening` and `closing` are unbalanced
///
/// Based on https://docs.rs/parse-hyperlinks/latest/parse_hyperlinks/fn.take_until_unbalanced.html
pub fn take_until_unbalanced(opening: char, closing: char) -> impl Fn(&str) -> IResult<&str, &str> {
    use nom::error::ParseError;

    move |input: &str| {
        let mut index = 0;
        let mut bracket_counter = 0;
        while let Some(n) = &input[index..].find(&[opening, closing, '\\'][..]) {
            index += n;
            let mut it = input[index..].chars();
            match it.next() {
                Some(c) if c == '\\' => {
                    // Skip the escape char `\`.
                    index += '\\'.len_utf8();
                    // Skip also the following char.
                    if let Some(c) = it.next() {
                        index += c.len_utf8();
                    }
                }
                Some(c) if c == opening => {
                    bracket_counter += 1;
                    index += opening.len_utf8();
                }
                Some(c) if c == closing => {
                    bracket_counter -= 1;
                    index += closing.len_utf8();
                }
                _ => unreachable!(),
            };
            // We found the unmatched closing.
            if bracket_counter == -1 {
                //Do not consume it.
                index -= closing.len_utf8();
                return Ok((&input[index..], &input[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok(("", input))
        } else {
            Err(Err::Error(Error::from_error_kind(
                input,
                ErrorKind::TakeUntil,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_spans() {
        assert_eq!(
            styled_inline(r#"[some string content]{text-red-300}"#)
                .unwrap()
                .1,
            Inline::StyledInline(StyledInline {
                code: "text-red-300".into(),
                content: vec![t("some string content")],
                ..Default::default()
            })
        );

        assert_eq!(
            styled_inline(r#"[content]{color:red}{css}"#).unwrap().1,
            Inline::StyledInline(StyledInline {
                content: vec![t("content")],
                code: "color:red".into(),
                style_language: Some(String::from("css")),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_parameters() {
        assert_eq!(
            parameter(r#"&[name]{}"#).unwrap().1,
            Inline::Parameter(Parameter {
                name: "name".to_string(),
                ..Default::default()
            })
        );

        assert_eq!(
            parameter(r#"&[name]{bool}"#).unwrap().1,
            Inline::Parameter(Parameter {
                name: "name".to_string(),
                options: Box::new(ParameterOptions {
                    validator: Some(Validator::BooleanValidator(BooleanValidator::default())),
                    ..Default::default()
                }),
                ..Default::default()
            })
        );

        assert_eq!(
            parameter(r#"&[name_with_under7scoresAnd_digits_3]{}"#)
                .unwrap()
                .1,
            Inline::Parameter(Parameter {
                name: "name_with_under7scoresAnd_digits_3".to_string(),
                ..Default::default()
            })
        );
    }
}
