//! Parsing functions shared by `inlines.rs` and `blocks.rs`

use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, tag_no_case, take_while_m_n},
    character::{
        complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1, none_of},
        is_digit,
    },
    combinator::{map, map_res, opt, peek, recognize},
    multi::{many0, many0_count, many1, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use codec::{
    common::eyre::Result,
    schema::{Date, DateTime, Duration, Node, Time, Timestamp},
};
use codec_json5_trait::Json5Codec;
use codec_text_trait::TextCodec;

/// Parse a symbol (e.g. the name of a `Parameter` or `CallArgument`)
///
/// Will only recognize names that are valid (in most programming languages). An alternative is to be more
/// permissive here and to check validity of symbol names elsewhere.
pub fn symbol(input: &str) -> IResult<&str, String> {
    map_res(
        recognize(tuple((
            many1(alt((alpha1, tag("_")))),
            many0(alt((alphanumeric1, tag("_")))),
        ))),
        |str: &str| -> Result<String> { Ok(str.to_string()) },
    )(input)
}

/// Parse attributes inside curly braces
///
/// Curly braced attributes are used to specify options on various inline
/// attributes.
///
/// This is lenient to the form of attributes and consumes everything
/// until the closing bracket. Attribute names are converted to snake_case
/// (so that users don't have to remember which case to use).
pub fn curly_attrs(input: &str) -> IResult<&str, Vec<(String, Option<Node>)>> {
    alt((
        map(tag("{}"), |_| Vec::new()),
        delimited(
            terminated(char('{'), multispace0),
            separated_list0(multispace1, curly_attr),
            preceded(multispace0, char('}')),
        ),
    ))(input)
}

/// Parse an attribute inside a curly braced attributes into a string/node pair
///
/// Attributes can be single values (i.e. flags) or key-value pairs (separated
/// by `=` or `:`).
pub fn curly_attr(input: &str) -> IResult<&str, (String, Option<Node>)> {
    map_res(
        alt((
            map(
                tuple((
                    curly_attr_name,
                    tuple((multispace0, alt((tag("="), tag(":"))), multispace0)),
                    alt((primitive_node, unquoted_string_node)),
                )),
                |(name, _s, value)| (name, Some(value)),
            ),
            map(curly_attr_name, |name| (name, None)),
        )),
        |(name, value): (&str, Option<Node>)| -> Result<(String, Option<Node>)> {
            // Previously this used snake case by that was problematic for attributes such as "json5"
            // (got converted to "json_5"). Instead, we replace dashes with underscores.
            Ok((name.replace('-', "_"), value))
        },
    )(input)
}

/// Parse a name of curly braces attribute
pub fn curly_attr_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

/// Parse a true/false (case-insensitive) into a `Boolean` node
fn boolean_node(input: &str) -> IResult<&str, Node> {
    map_res(
        alt((tag_no_case("true"), tag_no_case("false"))),
        |str: &str| -> Result<Node> { Ok(Node::Boolean(str == "true")) },
    )(input)
}

/// Parse one or more digits into an `Integer` node
fn integer_node(input: &str) -> IResult<&str, Node> {
    map_res(
        // The peek avoids a float input being partially parsed as an integer
        // There is probably a better way/place to do this.
        tuple((opt(tag("-")), digit1, peek(is_not(".")))),
        |(sign, digits, _peek): (Option<&str>, &str, _)| -> Result<Node> {
            Ok(Node::Integer(
                (sign.unwrap_or_default().to_string() + digits).parse()?,
            ))
        },
    )(input)
}

/// Parse one or more digits into an `Number` node
fn number_node(input: &str) -> IResult<&str, Node> {
    map_res(double, |num| -> Result<Node> { Ok(Node::Number(num)) })(input)
}

/// Parse a single quoted string into a `String` node
fn single_quoted_string_node(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\'"), '\\', char('\''));
    let empty = tag("");
    delimited(char('\''), alt((escaped, empty)), char('\''))(input)
}

/// Parse a double quoted string into a `String` node
fn double_quoted_string_node(input: &str) -> IResult<&str, &str> {
    let escaped = escaped(none_of("\\\""), '\\', char('"'));
    let empty = tag("");
    delimited(char('"'), alt((escaped, empty)), char('"'))(input)
}

/// Parse characters into string into a `String` node
///
/// Excludes character that may be significant in places that this is used.
fn unquoted_string_node(input: &str) -> IResult<&str, Node> {
    map_res(is_not(" }"), |str: &str| -> Result<Node> {
        Ok(Node::String(str.to_string()))
    })(input)
}

/// Parse a single or double quoted string into a `String` node
fn string_node(input: &str) -> IResult<&str, Node> {
    map_res(
        alt((single_quoted_string_node, double_quoted_string_node)),
        |str: &str| -> Result<Node> { Ok(Node::String(str.to_string())) },
    )(input)
}

/// Parse a YYYY-mm-dd date
fn date_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(tuple((
            take_while_m_n(4, 4, |c| is_digit(c as u8)),
            char('-'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char('-'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
        ))),
        |str: &str| -> Result<Node> { Ok(Node::Date(Date::new(str.to_string()))) },
    )(input)
}

/// Parse a HH::MM::SS time
fn time_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(tuple((
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char(':'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
            char(':'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
        ))),
        |str: &str| -> Result<Node> { Ok(Node::Time(Time::new(str.to_string()))) },
    )(input)
}

/// Parse a YYYY-mm-ddTHH::MM::SS datetime
fn datetime_node(input: &str) -> IResult<&str, Node> {
    map_res(
        recognize(terminated(
            tuple((date_node, char('T'), time_node)),
            opt(char('Z')),
        )),
        |str: &str| -> Result<Node> { Ok(Node::DateTime(DateTime::new(str.to_string()))) },
    )(input)
}

/// Parse a JSON5-style square bracketed array into an `Array` node
///
/// Inner closing brackets can be escaped.
fn array_node(input: &str) -> IResult<&str, Node> {
    let escaped = escaped(none_of("\\]"), '\\', tag("]"));
    let empty = tag("");
    map_res(
        delimited(tag("["), alt((escaped, empty)), tag("]")),
        |inner: &str| -> Result<Node> { Node::from_json5(&["[", inner, "]"].concat()) },
    )(input)
}

/// Parse a JSON5-style curly braced object into an `Object` node
///
/// Inner closing braces can be escaped.
fn object_node(input: &str) -> IResult<&str, Node> {
    let escaped = escaped(none_of("\\}"), '\\', tag("}"));
    let empty = tag("");
    map_res(
        delimited(tag("{"), alt((escaped, empty)), tag("}")),
        |inner: &str| -> Result<Node> { Node::from_json5(&["{", inner, "}"].concat()) },
    )(input)
}

/// Any primitive node
fn primitive_node(input: &str) -> IResult<&str, Node> {
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
    ))(input)
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

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_single_quoted() {
        let (_, res) = single_quoted_string_node(r#"' \' 🤖 '"#).unwrap();
        assert_eq!(res, r#" \' 🤖 "#);
        let (_, res) = single_quoted_string_node("' → x'").unwrap();
        assert_eq!(res, " → x");
        let (_, res) = single_quoted_string_node("'  '").unwrap();
        assert_eq!(res, "  ");
        let (_, res) = single_quoted_string_node("''").unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_double_quoted() {
        let (_, res) = double_quoted_string_node(r#""a\"b""#).unwrap();
        assert_eq!(res, r#"a\"b"#);
        let (_, res) = double_quoted_string_node(r#"" \" 🤖 ""#).unwrap();
        assert_eq!(res, r#" \" 🤖 "#);
        let (_, res) = double_quoted_string_node(r#"" → x""#).unwrap();
        assert_eq!(res, " → x");
        let (_, res) = double_quoted_string_node(r#""  ""#).unwrap();
        assert_eq!(res, "  ");
        let (_, res) = double_quoted_string_node(r#""""#).unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_square_bracketed() -> Result<()> {
        let (_, res) = array_node("[1,2,3]")?;
        assert_eq!(res, Node::from_json5("[1, 2, 3]")?);

        let (_, res) = array_node("['a', 'b']").unwrap();
        assert_eq!(res, Node::from_json5(r#"["a", "b"]"#)?);

        let (_, res) = array_node("[\"string \\] with closing bracket\"]").unwrap();
        assert_eq!(
            res,
            Node::from_json5(r#"["string ] with closing bracket"]"#)?
        );

        Ok(())
    }

    #[test]
    fn test_curly_attrs() -> Result<()> {
        assert_eq!(
            curly_attrs(r#"{a}"#).unwrap().1,
            vec![("a".to_string(), None),]
        );

        assert_eq!(
            curly_attrs(r#"{a=1 b='2' c:-3 d = 4.0}"#)?.1,
            vec![
                ("a".to_string(), Some(Node::Integer(1))),
                ("b".to_string(), Some(Node::String("2".to_string()))),
                ("c".to_string(), Some(Node::Integer(-3))),
                ("d".to_string(), Some(Node::Number(4.0)))
            ]
        );

        assert_eq!(
            curly_attrs(r#"{date min=2022-09-01 max='2022-09-30' def="2022-09-15"}"#)?.1,
            vec![
                ("date".to_string(), None),
                (
                    "min".to_string(),
                    Some(Node::Date(Date::new("2022-09-01".to_string())))
                ),
                (
                    "max".to_string(),
                    Some(Node::String("2022-09-30".to_string()))
                ),
                (
                    "def".to_string(),
                    Some(Node::String("2022-09-15".to_string()))
                ),
            ]
        );

        // Multiple spaces are fine
        assert_eq!(
            curly_attrs(r#"{   a     b=21 c : 1.234 d="   Internal  spaces "  }"#)?.1,
            vec![
                ("a".to_string(), None),
                ("b".to_string(), Some(Node::Integer(21))),
                ("c".to_string(), Some(Node::Number(1.234))),
                (
                    "d".to_string(),
                    Some(Node::String("   Internal  spaces ".to_string()))
                )
            ]
        );

        Ok(())
    }
}
