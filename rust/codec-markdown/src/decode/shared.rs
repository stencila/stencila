//! Parsing functions shared by `inlines.rs` and `blocks.rs`

use std::str::FromStr;

use markdown::mdast;
use winnow::{
    ascii::{dec_int, digit1, float, multispace0, multispace1, take_escaped, Caseless},
    combinator::{alt, delimited, not, opt, peek, separated, separated_pair, terminated},
    error::{ErrMode, ErrorKind, ParserError},
    stream::Stream,
    token::{none_of, take_while},
    Located, PResult, Parser,
};

use codec::schema::{
    Date, DateTime, Duration, ExecutionMode, ImageObject, InstructionMessage, InstructionType,
    MessagePart, Node, Time, Timestamp,
};
use codec_json5_trait::Json5Codec;
use codec_text_trait::TextCodec;

use crate::decode::inlines::mds_to_string;

/// Parse a name (e.g. name of a variable, parameter, call argument, or curly braced option)
///
/// Will only recognize names that are valid in (most) programming languages.
pub(super) fn name<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    (
        take_while(1.., |c: char| c.is_ascii_alphabetic() || c == '_'),
        take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '_'),
    )
        .take()
        .parse_next(input)
}

/// Parse a execution mode
pub(super) fn execution_mode(input: &mut &str) -> PResult<ExecutionMode> {
    alt(("always", "auto", "locked", "lock"))
        .map(|typ| match typ {
            "always" => ExecutionMode::Always,
            "auto" => ExecutionMode::Auto,
            "locked" | "lock" => ExecutionMode::Locked,
            _ => unreachable!(),
        })
        .parse_next(input)
}

/// Parse an instruction type
pub(super) fn instruction_type(input: &mut Located<&str>) -> PResult<InstructionType> {
    alt(("new", "edit", "fix", "describe"))
        .map(|typ| match typ {
            "new" => InstructionType::New,
            "edit" => InstructionType::Edit,
            "fix" => InstructionType::Fix,
            "describe" => InstructionType::Describe,
            _ => unreachable!(),
        })
        .parse_next(input)
}

/// Parse instruction options
pub(super) fn instruction_options<'s>(input: &mut Located<&'s str>) -> PResult<Vec<&'s str>> {
    separated(
        0..,
        alt((
            "run",
            "!run",
            (alt(('x', 'y', 'q', 's', 'c', 't')), digit1).take(),
        )),
        multispace1,
    )
    .parse_next(input)
}

/// Parse the name of a prompt of an instruction (e.g. `insert-image-object`, `joe@example.org`)
pub(super) fn prompt<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    (
        take_while(1.., |c: char| c.is_ascii_alphabetic()),
        take_while(0.., |c: char| {
            c.is_ascii_alphanumeric() || "_-/.@".contains(c)
        }),
    )
        .take()
        .parse_next(input)
}

/// Parse the name of a model of an instruction (e.g. `openai/gpt4`)
pub(super) fn model<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    (
        take_while(1.., |c: char| c.is_ascii_alphabetic()),
        take_while(0.., |c: char| {
            c.is_ascii_alphanumeric() || "_-/".contains(c)
        }),
    )
        .take()
        .parse_next(input)
}

/// Take characters until `opening` and `closing` are unbalanced
pub(super) fn take_until_unbalanced<'s>(
    opening: char,
    closing: char,
) -> impl Fn(&mut Located<&'s str>) -> PResult<&'s str> {
    move |input: &mut Located<&'s str>| {
        let mut counter = 0;
        for (index, char) in input.char_indices() {
            if char == opening {
                counter += 1
            } else if char == closing {
                counter -= 1
            }

            if counter < 0 {
                return Ok(input.next_slice(index));
            }
        }
        Ok(input.next_slice(input.len()))
    }
}

/// Parse "curly braced attrs" (options inside curly braces)
///
/// Curly braced options are used to specify options on various
/// node types. Separated by whitespace with optional commas
pub(super) fn attrs<'s>(input: &mut Located<&'s str>) -> PResult<Vec<(&'s str, Option<Node>)>> {
    delimited(
        ('{', multispace0),
        separated(
            0..,
            attr,
            alt(((multispace0, ',', multispace0).take(), multispace1)),
        ),
        (multispace0, '}'),
    )
    .parse_next(input)
}

/// Parse a single attr inside `attrs`
///
/// Attributes can be single values (i.e. flags) or key-value pairs (separated by `=`).
pub(super) fn attr<'s>(input: &mut Located<&'s str>) -> PResult<(&'s str, Option<Node>)> {
    alt((
        separated_pair(
            name,
            (multispace0, '=', multispace0),
            alt((primitive_node, unquoted_string_node)),
        )
        .map(|(name, value)| (name, Some(value))),
        name.map(|name| (name, None)),
    ))
    .parse_next(input)
}

/// Any primitive node
pub(super) fn primitive_node(input: &mut Located<&str>) -> PResult<Node> {
    alt((
        object_node,
        array_node,
        datetime_node,
        date_node,
        time_node,
        string_node,
        integer_node,
        number_node,
        boolean_node,
    ))
    .parse_next(input)
}

/// Parse a true/false (case-insensitive) into a `Boolean` node
fn boolean_node(input: &mut Located<&str>) -> PResult<Node> {
    alt((Caseless("true"), Caseless("false")))
        .map(|str: &str| Node::Boolean(str.to_lowercase() == "true"))
        .parse_next(input)
}

/// Parse an `Integer` node
fn integer_node(input: &mut Located<&str>) -> PResult<Node> {
    (dec_int, peek(not(".")))
        .map(|(num, ..)| Node::Integer(num))
        .parse_next(input)
}

/// Parse a `Number` node
fn number_node(input: &mut Located<&str>) -> PResult<Node> {
    float.map(Node::Number).parse_next(input)
}

/// Parse a single or double quoted string into a `String` node
fn string_node(input: &mut Located<&str>) -> PResult<Node> {
    alt((single_quoted_string_node, double_quoted_string_node)).parse_next(input)
}

/// Parse a single quoted string into a `String` node
fn single_quoted_string_node(input: &mut Located<&str>) -> PResult<Node> {
    delimited('\'', take_escaped(none_of(['\\', '\'']), '\\', '\''), '\'')
        .map(|value: &str| Node::String(value.to_string()))
        .parse_next(input)
}

/// Parse a double quoted string into a `String` node
fn double_quoted_string_node(input: &mut Located<&str>) -> PResult<Node> {
    delimited('"', take_escaped(none_of(['\\', '"']), '\\', '"'), '"')
        .map(|value: &str| Node::String(value.to_string()))
        .parse_next(input)
}

/// Parse characters into string into a `String` node
///
/// Excludes character that may be significant in places that this is used.
fn unquoted_string_node(input: &mut Located<&str>) -> PResult<Node> {
    take_while(1.., |c: char| c != ' ' && c != '}')
        .take()
        .map(|value: &str| Node::String(value.to_string()))
        .parse_next(input)
}

/// Parse a YYYY-mm-ddTHH::MM::SS datetime
fn datetime_node(input: &mut Located<&str>) -> PResult<Node> {
    terminated((date_node, 'T', time_node), opt('Z'))
        .take()
        .map(|str: &str| Node::DateTime(DateTime::new(str.to_string())))
        .parse_next(input)
}

fn digits4<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    take_while(4..=4, '0'..='9').parse_next(input)
}

fn digits2<'s>(input: &mut Located<&'s str>) -> PResult<&'s str> {
    take_while(2..=2, '0'..='9').parse_next(input)
}

/// Parse a YYYY-mm-dd date
fn date_node(input: &mut Located<&str>) -> PResult<Node> {
    (digits4, '-', digits2, '-', digits2)
        .take()
        .map(|str: &str| Node::Date(Date::new(str.to_string())))
        .parse_next(input)
}

/// Parse a HH::MM::SS time
fn time_node(input: &mut Located<&str>) -> PResult<Node> {
    (digits2, ':', digits2, ':', digits2)
        .take()
        .map(|str: &str| Node::Time(Time::new(str.to_string())))
        .parse_next(input)
}

/// Parse a JSON5-style square bracketed array into an `Array` node
fn array_node(input: &mut Located<&str>) -> PResult<Node> {
    let json5 = ('[', take_until_unbalanced('[', ']'), ']')
        .take()
        .parse_next(input)?;
    Node::from_json5(json5).map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify))
}

/// Parse a JSON5-style curly braced object into an `Object` node
fn object_node(input: &mut Located<&str>) -> PResult<Node> {
    let json5 = ('{', take_until_unbalanced('{', '}'), '}')
        .take()
        .parse_next(input)?;
    Node::from_json5(json5).map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify))
}

/// Convert a [`Node`] to a `String`
pub fn node_to_string(node: Node) -> String {
    node.to_text().0
}

/// Convert a [`Node`] to a type that has [`FromStr`] implemented
pub fn node_to_from_str<T: FromStr>(node: Node) -> Option<T> {
    T::from_str(&node_to_string(node)).ok()
}

/// Convert a [`Node`] to a `f64`
pub fn node_to_option_number(node: Node) -> Option<f64> {
    match node {
        Node::Number(num) => Some(num),
        Node::Integer(num) => Some(num as f64),
        _ => node_to_from_str::<f64>(node),
    }
}

/// Convert a [`Node`] to a `i64`
pub fn node_to_option_i64(node: Node) -> Option<i64> {
    match node {
        Node::Integer(int) => Some(int),
        _ => node_to_from_str::<i64>(node),
    }
}

/// Convert a [`Node`] to a [`Date`] if possible
pub fn node_to_option_date(node: Node) -> Option<Date> {
    match node {
        Node::Date(date) => Some(date),
        Node::String(string) => Some(Date::new(string)),
        _ => None,
    }
}

/// Convert a [`Node`] to a [`Time`] if possible
pub fn node_to_option_time(node: Node) -> Option<Time> {
    match node {
        Node::Time(time) => Some(time),
        Node::String(string) => Some(Time::new(string)),
        _ => None,
    }
}

/// Convert a [`Node`] to a [`DateTime`] if possible
pub fn node_to_option_datetime(node: Node) -> Option<DateTime> {
    match node {
        Node::DateTime(datetime) => Some(datetime),
        Node::String(string) => Some(DateTime::new(string)),
        _ => None,
    }
}

/// Convert a [`Node`] to a [`Timestamp`] if possible
pub fn node_to_option_timestamp(node: Node) -> Option<Timestamp> {
    match node {
        Node::Timestamp(timestamp) => Some(timestamp),
        _ => None,
    }
}

/// Convert a [`Node`] to a [`Duration`] if possible
pub fn node_to_option_duration(node: Node) -> Option<Duration> {
    match node {
        Node::Duration(duration) => Some(duration),
        _ => None,
    }
}

/// Parse a string into an [`InstructionMessage`]
///
/// Parses the string as Markdown and splits images into separate
/// message parts.
pub fn string_to_instruction_message(md: &str) -> InstructionMessage {
    use markdown::{to_mdast, ParseOptions};
    use mdast::Node;

    let Ok(Node::Root(root)) = to_mdast(md, &ParseOptions::default()) else {
        return InstructionMessage::from(md);
    };

    let Some(Node::Paragraph(mdast::Paragraph { children, .. })) = root.children.first().cloned()
    else {
        return InstructionMessage::from(md);
    };

    let mut parts = Vec::with_capacity(1);

    let mut text = String::new();
    for node in children {
        match node {
            Node::Image(image) => {
                if !text.is_empty() {
                    parts.push(MessagePart::from(text.drain(..)))
                }
                let content_url = if image.url.starts_with("file://")
                    || image.url.starts_with("https://")
                    || image.url.starts_with("http://")
                    || image.url.starts_with("data:image/")
                {
                    image.url
                } else {
                    ["file://", &image.url].concat()
                };

                parts.push(MessagePart::ImageObject(ImageObject {
                    content_url,
                    ..Default::default()
                }))
            }
            Node::Text(node) => {
                text += &node.value;
            }
            _ => {
                text += &mds_to_string(&[node]);
            }
        }
    }
    if !text.is_empty() {
        parts.push(MessagePart::from(text.drain(..)))
    }

    InstructionMessage {
        parts,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use codec::common::eyre::Result;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_single_quoted_string_node() -> Result<()> {
        assert_eq!(
            single_quoted_string_node(&mut Located::new(r#"' \' abc'"#)).unwrap(),
            Node::String(r#" \' abc"#.to_string())
        );

        assert_eq!(
            single_quoted_string_node(&mut Located::new("'  '")).unwrap(),
            Node::String("  ".to_string())
        );

        assert_eq!(
            single_quoted_string_node(&mut Located::new("''")).unwrap(),
            Node::String("".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_double_quoted_string_node() -> Result<()> {
        assert_eq!(
            double_quoted_string_node(&mut Located::new(r#"" \" abc""#)).unwrap(),
            Node::String(r#" \" abc"#.to_string())
        );

        assert_eq!(
            double_quoted_string_node(&mut Located::new(r#""  ""#)).unwrap(),
            Node::String("  ".to_string())
        );

        assert_eq!(
            double_quoted_string_node(&mut Located::new(r#""""#)).unwrap(),
            Node::String("".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_array_node() -> Result<()> {
        assert_eq!(
            array_node(&mut Located::new("[1,2,3]")).unwrap(),
            Node::from_json5("[1, 2, 3]")?
        );

        assert_eq!(
            array_node(&mut Located::new("['a', 'b']")).unwrap(),
            Node::from_json5(r#"["a", "b"]"#)?
        );

        assert_eq!(
            array_node(&mut Located::new("[1, [2, 3]]")).unwrap(),
            Node::from_json5("[1, [2, 3]]")?
        );

        Ok(())
    }

    #[test]
    fn test_object_node() -> Result<()> {
        assert_eq!(
            object_node(&mut Located::new("{a: 1}")).unwrap(),
            Node::from_json5("{a: 1}")?
        );

        assert_eq!(
            object_node(&mut Located::new("{a: {ab: 1, abc: [1, 2, 3]}}")).unwrap(),
            Node::from_json5("{a: {ab: 1, abc: [1, 2, 3]}}")?
        );

        Ok(())
    }

    #[test]
    fn test_attrs() -> Result<()> {
        assert_eq!(
            attrs(&mut Located::new(r#"{a}"#)).unwrap(),
            vec![("a", None)]
        );

        assert_eq!(
            attrs(&mut Located::new(r#"{a=true}"#)).unwrap(),
            vec![("a", Some(Node::Boolean(true)))]
        );

        assert_eq!(
            attrs(&mut Located::new(r#"{a=true b=123}"#)).unwrap(),
            vec![
                ("a", Some(Node::Boolean(true))),
                ("b", Some(Node::Integer(123)))
            ]
        );

        assert_eq!(
            attrs(&mut Located::new(r#"{a=1.23 b='b' c="c"}"#)).unwrap(),
            vec![
                ("a", Some(Node::Number(1.23))),
                ("b", Some(Node::String("b".to_string()))),
                ("c", Some(Node::String("c".to_string())))
            ]
        );

        assert_eq!(
            attrs(&mut Located::new(r#"{a=1, b='2' ,c=-3 ,  d = 4.0}"#)).unwrap(),
            vec![
                ("a", Some(Node::Integer(1))),
                ("b", Some(Node::String("2".to_string()))),
                ("c", Some(Node::Integer(-3))),
                ("d", Some(Node::Number(4.0)))
            ]
        );

        assert_eq!(
            attrs(&mut Located::new(
                r#"{date min=2022-09-01 max=2022-09-30 def=2022-09-15}"#
            ))
            .unwrap(),
            vec![
                ("date", None),
                ("min", Some(Node::Date(Date::new("2022-09-01".to_string())))),
                ("max", Some(Node::Date(Date::new("2022-09-30".to_string())))),
                ("def", Some(Node::Date(Date::new("2022-09-15".to_string())))),
            ]
        );

        assert_eq!(
            attrs(&mut Located::new(r#"{time min=00:11:22}"#)).unwrap(),
            vec![
                ("time", None),
                ("min", Some(Node::Time(Time::new("00:11:22".to_string())))),
            ]
        );

        assert_eq!(
            attrs(&mut Located::new(
                r#"{   a     b=21 c = 1.234 d="   Internal  spaces "  }"#
            ))
            .unwrap(),
            vec![
                ("a", None),
                ("b", Some(Node::Integer(21))),
                ("c", Some(Node::Number(1.234))),
                ("d", Some(Node::String("   Internal  spaces ".to_string())))
            ]
        );

        Ok(())
    }

    #[test]
    fn test_take_until_unbalanced() {
        assert_eq!(
            take_until_unbalanced('{', '}')
                .parse_next(&mut Located::new("abc }"))
                .unwrap(),
            "abc "
        );

        assert_eq!(
            take_until_unbalanced('{', '}')
                .parse_next(&mut Located::new("a{{b}c}} foo"))
                .unwrap(),
            "a{{b}c}"
        );

        assert_eq!(
            ('{', take_until_unbalanced('{', '}'), '}')
                .take()
                .parse_next(&mut Located::new("{a:1, b:2}"))
                .unwrap(),
            "{a:1, b:2}"
        );
    }
}
