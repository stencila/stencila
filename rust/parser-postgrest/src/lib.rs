use std::path::Path;

use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case, take_while1},
    character::{
        complete::{char, multispace0, multispace1},
        is_alphanumeric,
    },
    combinator::{all_consuming, map, opt, peek, recognize},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;
use nom_recursive::{recursive_parser, RecursiveInfo};

use parser::{
    common::{
        eyre::{bail, Result},
        itertools::Itertools,
    },
    formats::Format,
    graph_triples::{resources::ResourceDigest, Resource, ResourceInfo},
    utils::{parse_file_interps, parse_var_interps},
    Parser, ParserTrait,
};

/// A parser for PostgREST expressions
pub struct PostgrestParser;

impl ParserTrait for PostgrestParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Postgrest,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let (http, syntax_errors) = match transpile(code) {
            Ok(http) => (http, Some(false)),
            Err(..) => (String::new(), Some(true)),
        };

        let relations = [parse_var_interps(code, path), parse_file_interps(&http)].concat();

        let compile_digest = ResourceDigest::from_strings(code, None);

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            None,
            syntax_errors,
            Some(compile_digest),
            None,
            None,
        );

        Ok(resource_info)
    }
}

pub fn transpile(code: &str) -> Result<String> {
    match all_consuming(alt((select, insert_upsert, update, delete, call)))(span(code)) {
        Ok((.., http)) => Ok(http),
        Err(error) => bail!("Error parsing PostgREST statement: {}", error),
    }
}

type Span<'a> = LocatedSpan<&'a str, RecursiveInfo>;

fn span(s: &str) -> Span {
    LocatedSpan::new_extra(s, RecursiveInfo::new())
}

fn select(s: Span) -> IResult<Span, String> {
    alt((
        map(
            preceded(
                terminated(tag_no_case("from"), multispace1),
                pair(
                    table,
                    opt(preceded(
                        multispace1,
                        separated_list0(multispace1, alt((select_clause, filter_clause))),
                    )),
                ),
            ),
            |(table, options)| {
                [
                    "GET /",
                    &table,
                    &options
                        .map(|options| ["?", &options.join("&")].concat())
                        .unwrap_or_default(),
                    " HTTP/1.1",
                ]
                .concat()
            },
        ),
        map(
            tuple((
                select_clause,
                preceded(
                    delimited(multispace1, tag_no_case("from"), multispace1),
                    table,
                ),
                opt(preceded(multispace1, filter_clause)),
            )),
            |(select, table, filter)| {
                let mut params = String::new();
                if !select.is_empty() {
                    params.push_str(&select);
                }
                if let Some(filter) = filter {
                    if !params.is_empty() {
                        params.push('&');
                    }
                    params.push_str(&filter);
                }

                ["GET /", &table, "?", &params, " HTTP/1.1"].concat()
            },
        ),
    ))(s)
}

fn insert_upsert(s: Span) -> IResult<Span, String> {
    map(
        tuple((
            alt((tag_no_case("insert"), tag_no_case("upsert"))),
            preceded(multispace1, table),
            opt(preceded(
                multispace1,
                separated_list1(multispace1, alt((columns_clause, format_clause))),
            )),
            preceded(multispace1, alt((file_clause, data_clause))),
        )),
        |(action, table, options, data): (Span, String, Option<Vec<String>>, String)| {
            let mut headers = if action.to_lowercase() == "upsert" {
                "Prefer: resolution=merge-duplicates\n"
            } else {
                ""
            }
            .to_string();

            let mut query = String::new();
            for option in options.unwrap_or_default() {
                if option.starts_with("Content-Type:") {
                    headers += &option;
                } else {
                    query = ["?", &option].concat();
                }
            }

            // If content type not specified by format or file extension then default to JSON5
            if !headers.contains("Content-Type:") && !data.contains("Content-Type:") {
                headers += "Content-Type: application/json5\n";
            }

            ["POST /", &table, &query, " HTTP/1.1\n", &headers, &data].concat()
        },
    )(s)
}

fn update(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            tag_no_case("update"),
            tuple((
                preceded(multispace1, table),
                preceded(multispace1, alt((filter_clause, filter_expression_url))),
                opt(preceded(
                    multispace1,
                    separated_list1(multispace1, alt((columns_clause, format_clause))),
                )),
                preceded(multispace1, alt((file_clause, data_clause))),
            )),
        ),
        |(table, filters, options, data): (String, String, Option<Vec<String>>, String)| {
            let mut query = ["?", &filters].concat();
            let mut headers = String::new();
            for option in options.unwrap_or_default() {
                if option.starts_with("Content-Type:") {
                    headers += &option;
                } else {
                    query += &["&", &option].concat();
                }
            }

            // If content type not specified by format or file extension then default to JSON5
            if !headers.contains("Content-Type:") && !data.contains("Content-Type:") {
                headers += "Content-Type: application/json5";
            }

            ["PATCH /", &table, &query, " HTTP/1.1\n", &headers, &data].concat()
        },
    )(s)
}

fn delete(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            terminated(tag_no_case("delete"), multispace1),
            separated_pair(
                is_not_space,
                multispace1,
                alt((filter_clause, filter_expression_url)),
            ),
        ),
        |(table, filters)| ["DELETE /", &table, "?", &filters, " HTTP/1.1"].concat(),
    )(s)
}

fn call(s: Span) -> IResult<Span, String> {
    map(
        preceded(terminated(tag_no_case("call"), multispace1), is_not_space),
        |func| ["POST /rpc/", &func, " HTTP/1.1"].concat(),
    )(s)
}

fn table(s: Span) -> IResult<Span, String> {
    map(is_not(" \t\n"), |chars: Span| chars.to_string())(s)
}

fn select_clause(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            terminated(tag_no_case("select"), multispace1),
            separated_list1(delimited(multispace0, char(','), multispace0), is_not(" ,")),
        ),
        |columns: Vec<Span>| {
            let columns = columns.iter().map(|column| column.to_string()).join(",");
            if columns == "*" {
                String::new()
            } else {
                ["select=", &columns].concat()
            }
        },
    )(s)
}

fn columns_clause(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            terminated(tag_no_case("columns"), multispace1),
            separated_list1(delimited(multispace0, char(','), multispace0), is_not(" ,")),
        ),
        |columns: Vec<Span>| {
            let columns = columns.iter().map(|column| column.to_string()).join(",");
            if columns == "*" {
                String::new()
            } else {
                ["columns=", &columns].concat()
            }
        },
    )(s)
}

fn format_clause(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            terminated(tag_no_case("format"), multispace1),
            take_while1(|c| is_alphanumeric(c as u8)),
        ),
        |format: Span| {
            let format = format.to_string();
            let content_type = match format.as_str() {
                "csv" => "text/csv".to_string(),
                "json" => "application/json".to_string(),
                "json5" => "application/json5".to_string(),
                _ => format,
            };
            ["Content-Type: ", &content_type, "\n"].concat()
        },
    )(s)
}

fn file_clause(s: Span) -> IResult<Span, String> {
    map(
        preceded(terminated(tag_no_case("file"), multispace1), is_not_space),
        |file| {
            [
                // Add a content type here to avoid the default to json5
                "Content-Type: ",
                if file.ends_with(".csv") {
                    "text/csv"
                } else {
                    "application/json"
                },
                "\n\n",
                "@{",
                &file,
                "}",
            ]
            .concat()
        },
    )(s)
}

fn data_clause(s: Span) -> IResult<Span, String> {
    map(is_not(""), |data: Span| {
        // Do not add a content type header here: `format` clause should be used, or defaults to json5
        ["\n\n", &data.to_string()].concat()
    })(s)
}

fn filter_clause(s: Span) -> IResult<Span, String> {
    map(
        preceded(
            terminated(
                alt((tag_no_case("filter"), tag_no_case("where"))),
                multispace1,
            ),
            filter_expression,
        ),
        |expr| expr.to_url(true, false),
    )(s)
}

fn filter_expression_url(s: Span) -> IResult<Span, String> {
    map(filter_expression, |expr| expr.to_url(true, false))(s)
}

#[derive(Debug, PartialEq, Eq)]
enum ExprSide {
    Empty,
    String(String),
    Expr(Box<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
struct Expr {
    left: ExprSide,
    op: String,
    right: ExprSide,
}

impl Expr {
    fn to_url(&self, top: bool, negate: bool) -> String {
        if let (ExprSide::Expr(left), "and", ExprSide::Expr(right)) =
            (&self.left, self.op.as_str(), &self.right)
        {
            if top && !negate {
                [&left.to_url(true, false), "&", &right.to_url(true, false)].concat()
            } else if top && negate {
                [
                    "not.and=(",
                    &left.to_url(false, false),
                    ",",
                    &right.to_url(false, false),
                    ")",
                ]
                .concat()
            } else {
                [
                    negate.then_some("not.").unwrap_or_default(),
                    "and(",
                    &left.to_url(false, false),
                    ",",
                    &right.to_url(false, false),
                    ")",
                ]
                .concat()
            }
        } else if let (ExprSide::Expr(left), "or", ExprSide::Expr(right)) =
            (&self.left, self.op.as_str(), &self.right)
        {
            [
                "or",
                if top { "=(" } else { "(" },
                &left.to_url(false, false),
                ",",
                &right.to_url(false, false),
                ")",
            ]
            .concat()
        } else if let ("not", ExprSide::Expr(right)) = (self.op.as_str(), &self.right) {
            right.to_url(top, true)
        } else if let ("()", ExprSide::Expr(right)) = (self.op.as_str(), &self.right) {
            right.to_url(top, negate)
        } else if let (ExprSide::String(left), ExprSide::String(right)) = (&self.left, &self.right)
        {
            [
                left.as_str(),
                if top { "=" } else { "." },
                negate.then_some("not.").unwrap_or_default(),
                self.op.as_str(),
                ".",
                right.as_str(),
            ]
            .concat()
        } else {
            format!("Not handled: {:?}", self)
        }
    }
}

#[recursive_parser]
fn filter_expression(s: Span) -> IResult<Span, Expr> {
    alt((
        or_expression,
        and_expression,
        paren_expression,
        not_expression,
        comparison_expression,
    ))(s)
}

#[recursive_parser]
fn or_expression(s: Span) -> IResult<Span, Expr> {
    map(
        tuple((
            alt((
                or_expression,
                and_expression,
                paren_expression,
                not_expression,
                comparison_expression,
            )),
            delimited(multispace1, tag_no_case("or"), multispace1),
            alt((
                or_expression,
                and_expression,
                paren_expression,
                not_expression,
                comparison_expression,
            )),
        )),
        |(left, tag, right)| Expr {
            left: ExprSide::Expr(Box::new(left)),
            op: tag.to_string(),
            right: ExprSide::Expr(Box::new(right)),
        },
    )(s)
}

#[recursive_parser]
fn and_expression(s: Span) -> IResult<Span, Expr> {
    map(
        tuple((
            alt((
                and_expression,
                paren_expression,
                not_expression,
                comparison_expression,
            )),
            delimited(multispace1, tag_no_case("and"), multispace1),
            alt((
                and_expression,
                paren_expression,
                not_expression,
                comparison_expression,
            )),
        )),
        |(left, tag, right)| Expr {
            left: ExprSide::Expr(Box::new(left)),
            op: tag.to_string(),
            right: ExprSide::Expr(Box::new(right)),
        },
    )(s)
}

fn not_expression(s: Span) -> IResult<Span, Expr> {
    map(
        preceded(
            terminated(tag_no_case("not"), alt((peek(is_a("(")), multispace1))),
            alt((
                or_expression,
                and_expression,
                paren_expression,
                comparison_expression,
            )),
        ),
        |right| Expr {
            left: ExprSide::Empty,
            op: "not".to_string(),
            right: ExprSide::Expr(Box::new(right)),
        },
    )(s)
}

#[recursive_parser]
fn paren_expression(s: Span) -> IResult<Span, Expr> {
    map(
        delimited(
            terminated(char('('), multispace0),
            alt((
                or_expression,
                and_expression,
                not_expression,
                comparison_expression,
            )),
            preceded(multispace0, char(')')),
        ),
        |expr| Expr {
            left: ExprSide::Empty,
            op: "()".to_string(),
            right: ExprSide::Expr(Box::new(expr)),
        },
    )(s)
}

fn comparison_expression(s: Span) -> IResult<Span, Expr> {
    map(
        tuple((
            is_not(" \t\n=><!~@&-()"), // Take until whitespace of a character involved in an operator is met
            delimited(multispace0, comparison_operator, multispace0),
            alt((
                recognize(tuple((char('('), is_not(")"), char(')')))),
                delimited(char('"'), is_not("\""), char('"')),
                delimited(char('\''), is_not("'"), char('\'')),
                is_not(" \t\n)"),
            )),
        )),
        |(column, operator, value)| Expr {
            left: ExprSide::String(column.to_string()),
            op: operator.to_string(),
            right: ExprSide::String(value.to_string()),
        },
    )(s)
}

fn comparison_operator(s: Span) -> IResult<Span, &str> {
    alt((
        // Note: order IS important to prevent "premature matching"
        // See https://postgrest.org/en/stable/api.html#operators
        fts, cs, cd, ov, sl, sr, nxr, nxl, adj, neq, eq, gte, gt, lte, lt, like, ilike, imatch,
        match_, in_, is,
    ))(s)
}

macro_rules! tag_op {
    ($tag:literal,$op:ident) => {
        fn $op(s: Span) -> IResult<Span, &str> {
            map(tag($tag), |_| stringify!($op))(s)
        }
    };
}

tag_op!("=", eq);
tag_op!(">", gt);
tag_op!(">=", gte);
tag_op!("<", lt);
tag_op!("<=", lte);
tag_op!("~*", imatch);
tag_op!("@@", fts);
tag_op!("@>", cs);
tag_op!("<@", cd);
tag_op!("&&", ov);
tag_op!("<<", sl);
tag_op!(">>", sr);
tag_op!("&<", nxr);
tag_op!("&>", nxl);
tag_op!("-|-", adj);

macro_rules! tag_no_case_op {
    ($tag:literal,$op:ident) => {
        fn $op(s: Span) -> IResult<Span, &str> {
            map(tag_no_case($tag), |_| stringify!($op))(s)
        }
    };
}

tag_no_case_op!("like", like);
tag_no_case_op!("ilike", ilike);
tag_no_case_op!("is", is);

fn neq(s: Span) -> IResult<Span, &str> {
    map(alt((tag("<>"), tag("!="))), |_| "neq")(s)
}

fn match_(s: Span) -> IResult<Span, &str> {
    map(char('~'), |_| "match")(s)
}

fn in_(s: Span) -> IResult<Span, &str> {
    map(tag_no_case("in"), |_| "in")(s)
}

fn is_not_space(s: Span) -> IResult<Span, String> {
    map(is_not(" \t\n"), |chars: Span| chars.to_string())(s)
}

#[cfg(test)]
mod tests {
    use test_utils::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_select() -> Result<()> {
        // Whole table
        assert_eq!(transpile("from table")?, "GET /table HTTP/1.1");
        assert_eq!(transpile("FROM\n   table")?, "GET /table HTTP/1.1");

        // Select column https://postgrest.org/en/stable/api.html#vertical-filtering-columns
        assert_eq!(
            transpile("from table select column")?,
            "GET /table?select=column HTTP/1.1"
        );
        // Whitespace between columns
        assert_eq!(
            transpile("FROM table\nSELECT column1,\tcolumn2, column3,\n  column4")?,
            "GET /table?select=column1,column2,column3,column4 HTTP/1.1"
        );
        // Renaming
        assert_eq!(
            transpile("from people select fullName:full_name, birthDate:birth_date")?,
            "GET /people?select=fullName:full_name,birthDate:birth_date HTTP/1.1"
        );
        // Casting
        assert_eq!(
            transpile("from people select full_name, salary::text")?,
            "GET /people?select=full_name,salary::text HTTP/1.1"
        );

        // JSON columns https://postgrest.org/en/stable/api.html#json-columns
        assert_eq!(
            transpile("from people\nselect id,\n  json_data->>blood_type,\n  json_data->phones")?,
            "GET /people?select=id,json_data->>blood_type,json_data->phones HTTP/1.1"
        );
        assert_eq!(
            transpile("from people select id,json_data->phones->0->>number")?,
            "GET /people?select=id,json_data->phones->0->>number HTTP/1.1"
        );

        // With resource embedding https://postgrest.org/en/stable/api.html#resource-embedding
        assert_eq!(
            transpile("from directors select last_name, films(title)")?,
            "GET /directors?select=last_name,films(title) HTTP/1.1"
        );
        assert_eq!(
            transpile("from actors select roles(character,films(title,year))")?,
            "GET /actors?select=roles(character,films(title,year)) HTTP/1.1"
        );

        // Alternative, SQL like syntax
        assert_eq!(
            transpile("select col1, col2 from table")?,
            "GET /table?select=col1,col2 HTTP/1.1"
        );
        assert_eq!(
            transpile("select col1, col2 from table where col3<10")?,
            "GET /table?select=col1,col2&col3=lt.10 HTTP/1.1"
        );
        assert_eq!(
            transpile("select * from table where col3<10")?,
            "GET /table?col3=lt.10 HTTP/1.1"
        );
        assert_eq!(
            transpile("from table where col3<10")?,
            "GET /table?col3=lt.10 HTTP/1.1"
        );

        Ok(())
    }

    #[test]
    fn test_filters() -> Result<()> {
        assert_eq!(
            transpile("from people filter age>=18")?,
            "GET /people?age=gte.18 HTTP/1.1"
        );

        assert_eq!(
            transpile("from people FILTER\nage >= 18")?,
            "GET /people?age=gte.18 HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter age in (18,19)")?,
            "GET /people?age=in.(18,19) HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter age=18 and student is true")?,
            "GET /people?age=eq.18&student=is.true HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter age!=18 or height<180")?,
            "GET /people?or=(age.neq.18,height.lt.180) HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter a=1 and b<2 and c>3")?,
            "GET /people?a=eq.1&b=lt.2&c=gt.3 HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter a=1 or b<2 or c>3")?,
            "GET /people?or=(a.eq.1,or(b.lt.2,c.gt.3)) HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter a=1 and b<2 or c>3")?,
            "GET /people?or=(and(a.eq.1,b.lt.2),c.gt.3) HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter a=1 and (b<2 or c>3)")?,
            "GET /people?a=eq.1&or=(b.lt.2,c.gt.3) HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter not a=1")?,
            "GET /people?a=not.eq.1 HTTP/1.1"
        );

        assert_eq!(
            transpile("from people filter not (a=1 and b>4)")?,
            "GET /people?not.and=(a.eq.1,b.gt.4) HTTP/1.1"
        );

        assert_eq!(
            // As above but not space after not
            transpile("from people filter not(a=1 and b>4)")?,
            "GET /people?not.and=(a.eq.1,b.gt.4) HTTP/1.1"
        );

        // Example from https://postgrest.org/en/stable/api.html#logical-operators
        assert_eq!(
            transpile("from people filter grade>=90 and student is true and (age=14 or not (age>=11 and age<=17))")?,
            "GET /people?grade=gte.90&student=is.true&or=(age.eq.14,not.and(age.gte.11,age.lte.17)) HTTP/1.1"
        );

        Ok(())
    }

    #[test]
    fn test_insert() -> Result<()> {
        assert_eq!(
            transpile("insert people {\"name\":\"Jaba\"}")?,
            "POST /people HTTP/1.1\nContent-Type: application/json5\n\n\n{\"name\":\"Jaba\"}"
        );

        assert_eq!(
            transpile("insert people format csv\n\nname\nJaba")?,
            "POST /people HTTP/1.1\nContent-Type: text/csv\n\n\nname\nJaba"
        );

        assert_eq!(
            transpile("insert people {name:'Jaba'}")?,
            "POST /people HTTP/1.1\nContent-Type: application/json5\n\n\n{name:'Jaba'}"
        );

        assert_eq!(
            transpile("insert people {name:$name}")?,
            "POST /people HTTP/1.1\nContent-Type: application/json5\n\n\n{name:$name}"
        );

        assert_eq!(
            transpile("insert people file people.json")?,
            "POST /people HTTP/1.1\nContent-Type: application/json\n\n@{people.json}"
        );

        assert_eq!(
            transpile("insert people file people.csv")?,
            "POST /people HTTP/1.1\nContent-Type: text/csv\n\n@{people.csv}"
        );

        assert_eq!(
            transpile("insert people columns name,age file people.csv")?,
            "POST /people?columns=name,age HTTP/1.1\nContent-Type: text/csv\n\n@{people.csv}"
        );

        assert_eq!(
            transpile("insert people file people.foo")?,
            "POST /people HTTP/1.1\nContent-Type: application/json\n\n@{people.foo}"
        );

        Ok(())
    }

    #[test]
    fn test_upsert() -> Result<()> {
        assert_eq!(
            transpile("upsert people {name:'Jaba'}")?,
            "POST /people HTTP/1.1\nPrefer: resolution=merge-duplicates\nContent-Type: application/json5\n\n\n{name:'Jaba'}"
        );

        assert_eq!(
            transpile("upsert people file people.json")?,
            "POST /people HTTP/1.1\nPrefer: resolution=merge-duplicates\nContent-Type: application/json\n\n@{people.json}"
        );

        Ok(())
    }

    #[test]
    fn test_update() -> Result<()> {
        assert_eq!(
            transpile("update people filter age >= 18 {age: 19}")?,
            "PATCH /people?age=gte.18 HTTP/1.1\nContent-Type: application/json5\n\n{age: 19}"
        );

        assert_eq!(
            transpile("update people filter age >= 18 columns age format csv\n\nage\n19")?,
            "PATCH /people?age=gte.18&columns=age HTTP/1.1\nContent-Type: text/csv\n\n\nage\n19"
        );

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<()> {
        assert_eq!(
            transpile("delete people filter age >= 18")?,
            "DELETE /people?age=gte.18 HTTP/1.1"
        );

        Ok(())
    }

    #[test]
    fn test_call() -> Result<()> {
        assert_eq!(
            transpile("call function_name")?,
            "POST /rpc/function_name HTTP/1.1"
        );

        Ok(())
    }

    #[test]
    fn test_paren_expression() {
        assert_eq!(
            paren_expression(span("(a<1)")).unwrap().1,
            Expr {
                left: ExprSide::Empty,
                op: "()".to_string(),
                right: ExprSide::Expr(Box::new(Expr {
                    left: ExprSide::String("a".to_string()),
                    op: "lt".to_string(),
                    right: ExprSide::String("1".to_string())
                }))
            }
        );
    }

    #[test]
    fn test_comparison_operator() {
        assert_eq!(comparison_operator(span("=")).unwrap().1, "eq");
        assert_eq!(comparison_operator(span(">")).unwrap().1, "gt");
        assert_eq!(comparison_operator(span(">=")).unwrap().1, "gte");
        assert_eq!(comparison_operator(span("<")).unwrap().1, "lt");
        assert_eq!(comparison_operator(span("<=")).unwrap().1, "lte");
        assert_eq!(comparison_operator(span("<>")).unwrap().1, "neq");
        assert_eq!(comparison_operator(span("!=")).unwrap().1, "neq");
        assert_eq!(comparison_operator(span("like")).unwrap().1, "like");
        assert_eq!(comparison_operator(span("ilike")).unwrap().1, "ilike");
        assert_eq!(comparison_operator(span("~")).unwrap().1, "match");
        assert_eq!(comparison_operator(span("~*")).unwrap().1, "imatch");
        assert_eq!(comparison_operator(span("in")).unwrap().1, "in");
        assert_eq!(comparison_operator(span("is")).unwrap().1, "is");
        assert_eq!(comparison_operator(span("@@")).unwrap().1, "fts");
        assert_eq!(comparison_operator(span("@>")).unwrap().1, "cs");
        assert_eq!(comparison_operator(span("<@")).unwrap().1, "cd");
        assert_eq!(comparison_operator(span("&&")).unwrap().1, "ov");
        assert_eq!(comparison_operator(span("<<")).unwrap().1, "sl");
        assert_eq!(comparison_operator(span(">>")).unwrap().1, "sr");
        assert_eq!(comparison_operator(span("&<")).unwrap().1, "nxr");
        assert_eq!(comparison_operator(span("&>")).unwrap().1, "nxl");
        assert_eq!(comparison_operator(span("-|-")).unwrap().1, "adj");
    }
}
