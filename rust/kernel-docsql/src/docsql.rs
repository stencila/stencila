use std::{
    fmt,
    sync::{Arc, LazyLock},
};

use itertools::Itertools;
use regex::{Captures, Regex};

use kernel_jinja::minijinja::{Environment, Error, ErrorKind, Value};

use crate::{CypherQuery, NodeProxies, NodeProxy, openalex::OpenAlexQuery};

/// Operator enum for filter operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operator {
    Eq,      // ==
    Neq,     // !=
    Lt,      // <
    Lte,     // <=
    Gt,      // >
    Gte,     // >=
    Match,   // ~=
    NoMatch, // ~!
    Starts,  // ^=
    Ends,    // $=
    In,      // in
    Has,     // has
}

impl Operator {
    /// Convert operator to its single character suffix
    pub fn to_suffix(self) -> &'static str {
        match self {
            Operator::Eq => "",
            Operator::Neq => "0",
            Operator::Lt => "1",
            Operator::Lte => "2",
            Operator::Gt => "3",
            Operator::Gte => "4",
            Operator::Match => "5",
            Operator::NoMatch => "6",
            Operator::Starts => "7",
            Operator::Ends => "8",
            Operator::In => "9",
            Operator::Has => "_",
        }
    }

    /// Parse a suffix character back to an operator
    pub fn from_suffix(c: char) -> Option<Self> {
        match c {
            '0' => Some(Operator::Neq),
            '1' => Some(Operator::Lt),
            '2' => Some(Operator::Lte),
            '3' => Some(Operator::Gt),
            '4' => Some(Operator::Gte),
            '5' => Some(Operator::Match),
            '6' => Some(Operator::NoMatch),
            '7' => Some(Operator::Starts),
            '8' => Some(Operator::Ends),
            '9' => Some(Operator::In),
            '_' => Some(Operator::Has),
            _ => None,
        }
    }

    /// Get the string representation of the operator
    pub fn as_str(&self) -> &'static str {
        match self {
            Operator::Eq => "==",
            Operator::Neq => "!=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Match => "~=",
            Operator::NoMatch => "~!",
            Operator::Starts => "^=",
            Operator::Ends => "$=",
            Operator::In => "in",
            Operator::Has => "has",
        }
    }

    /// Parse an operator string to enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" | "==" => Some(Operator::Eq),
            "!=" => Some(Operator::Neq),
            "<" => Some(Operator::Lt),
            "<=" => Some(Operator::Lte),
            ">" => Some(Operator::Gt),
            ">=" => Some(Operator::Gte),
            "~=" => Some(Operator::Match),
            "~!" => Some(Operator::NoMatch),
            "^=" => Some(Operator::Starts),
            "$=" => Some(Operator::Ends),
            "in" => Some(Operator::In),
            "has" => Some(Operator::Has),
            _ => None,
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The type of a property
///
/// Used to specify which operators can be used with it
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Boolean,
    Number,
    Date,
    Enum,
    String,
    Array,
}

impl PropertyType {
    /// Is an operator valid for a particular operator?
    pub fn is_valid(&self, op: Operator) -> bool {
        use Operator::*;
        match self {
            PropertyType::Boolean => matches!(op, Eq | Neq),
            PropertyType::Number => matches!(op, Eq | Neq | Lt | Lte | Gt | Gte | In),
            PropertyType::Date => matches!(op, Eq | Neq | Lt | Lte | Gt | Gte | In),
            PropertyType::Enum => matches!(op, Eq | Neq | In),
            PropertyType::String => !matches!(op, Has),
            PropertyType::Array => matches!(op, Has),
        }
    }
}

/// Names added to the Jinja environment with `env.add_global`
///
/// Maintaining this list is tedious. However, it is only
/// used as an optimization to avoid searching for
/// these function as variables in other kernels. As such it
/// does not need to be complete (or even accurate), but better
/// if it is (both complete and accurate).
///
/// Unfortunately, at this time, this can not be done dynamically.
pub(crate) const GLOBAL_NAMES: &[&str] = &[
    // Database names added in lib.rs
    "document",
    "workspace",
    "openalex",
    "github",
    // Added in cypher::add_document_functions
    // Static code
    "codeBlock",
    "codeBlocks",
    "codeInline",
    "codeInlines",
    // Executable code
    "codeChunk",
    "codeChunks",
    "chunk",
    "chunks",
    "codeExpression",
    "codeExpressions",
    "expression",
    "expressions",
    // Math
    "mathBlock",
    "mathBlocks",
    "mathInline",
    "mathInlines",
    // Media
    "image",
    "images",
    "audio",
    "audios",
    "video",
    "videos",
    // Containers
    "admonition",
    "admonitions",
    "claim",
    "claims",
    "heading",
    "headings",
    "list",
    "lists",
    "paragraph",
    "paragraphs",
    "section",
    "sections",
    "sentence",
    "sentences",
    // Metadata
    "organization",
    "organizations",
    "person",
    "people",
    "reference",
    "references",
    // Labelled types
    "figure",
    "figures",
    "table",
    "tables",
    "equation",
    "equations",
    // Section types
    "methods",
    "results",
    "introduction",
    "discussion",
    // Variables
    "variable",
    "variables",
    // Added in subquery::add_subquery_functions
    "_authors",
    "_owners",
    "_references",
    "_cites",
    "_citedBy",
    "_affiliations",
    "_organizations",
    "_topics",
    "_chunks",
    "_expressions",
    "_audios",
    "_images",
    "_videos",
    // GLOBAL_CONSTS added in add_constants
    "above",
    "below",
    "return",
    // Added in add_functions
    "combine",
];

/// Add global constants
pub(super) fn add_constants(env: &mut Environment) {
    for name in GLOBAL_CONSTS {
        env.add_global(*name, *name);
    }
}

pub(super) const GLOBAL_CONSTS: &[&str] = &["above", "below", "return"];

/// Add global functions to the environment
pub(super) fn add_functions(env: &mut Environment) {
    env.add_function("combine", combine);
}

/// Function to combine nodes from several queries
fn combine(args: &[Value]) -> Result<Value, Error> {
    let mut nodes = Vec::new();
    for value in args {
        if let Some(query) = value.downcast_object::<CypherQuery>() {
            nodes.append(&mut query.nodes());
        } else if let Some(query) = value.downcast_object::<OpenAlexQuery>() {
            nodes.append(&mut query.nodes());
        } else if let Some(proxies) = value.downcast_object::<NodeProxies>() {
            nodes.append(&mut proxies.nodes());
        } else if let Some(proxy) = value.downcast_object::<NodeProxy>() {
            nodes.append(&mut proxy.nodes());
        } else {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "all arguments should be queries or nodes resulting from queries",
            ));
        }
    }

    Ok(Value::from_object(NodeProxies::new(nodes, Arc::default())))
}

/// Strips comments after any `//`
///
/// Note that this will may result in blank lines which is
/// intentional for maintaining line numbers
pub(super) fn strip_comments(code: &str) -> String {
    code.lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .join("\n")
}

/// Encode DocsQL filter arguments into valid MiniJinja keyword arguments
///
/// Uses single digit codes and spacing to ensure that the code stays the same length.
pub(super) fn encode_filters(code: &str) -> String {
    static FILTERS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"((?:\(|,)\s*)(\*|(?:\.[a-zA-Z][\w_]*))\s*(==|\!=|<=|<|>=|>|\~=|\~\!|\^=|\$=|in|has|=)\s*")
            .expect("invalid regex")
    });

    let code = FILTERS.replace_all(code, |captures: &Captures| {
        let before = &captures[1];
        let property = &captures[2];
        let operator = &captures[3];

        let name = encode_filter(property, operator);

        let spaces = captures[0]
            .len()
            .saturating_sub(before.len() + name.len() + 1);
        let spaces = " ".repeat(spaces);

        [before, &name, &spaces, "="].concat()
    });

    static SUBQUERY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"((?:\(|,)\s*)\.\.\.([a-zA-Z][\w_]*)").expect("invalid regex")
    });

    SUBQUERY
        .replace_all(&code, |captures: &Captures| {
            let pre = &captures[1];
            let func = &captures[2];
            [pre, "_=_", func].concat()
        })
        .into()
}

/// Encode a property name and operator into a MiniJinja argument name
pub(super) fn encode_filter(property: &str, operator: &str) -> String {
    let name = match property {
        "*" => "_C", // Count
        _ => property.trim_start_matches("."),
    };

    let suffix = if let Some(op) = Operator::from_str(operator) {
        op.to_suffix()
    } else {
        operator
    };

    [name, suffix].concat()
}

/// Decode a MiniJinja argument name into a property name and operator
///
/// The inverse of `encode_filter`.
pub(super) fn decode_filter(arg_name: &str) -> (&str, Operator) {
    if arg_name.len() > 1 {
        if let Some(last_char) = arg_name.chars().last() {
            if let Some(op) = Operator::from_suffix(last_char) {
                let trimmed = &arg_name[..arg_name.len() - 1];
                (trimmed, op)
            } else {
                (arg_name, Operator::Eq)
            }
        } else {
            (arg_name, Operator::Eq)
        }
    } else {
        (arg_name, Operator::Eq)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn operator_conversions() {
        use super::{Operator, decode_filter};

        // All operators for testing
        let operators = [
            (Operator::Eq, "", "=="),
            (Operator::Neq, "0", "!="),
            (Operator::Lt, "1", "<"),
            (Operator::Lte, "2", "<="),
            (Operator::Gt, "3", ">"),
            (Operator::Gte, "4", ">="),
            (Operator::Match, "5", "~="),
            (Operator::NoMatch, "6", "~!"),
            (Operator::Starts, "7", "^="),
            (Operator::Ends, "8", "$="),
            (Operator::In, "9", "in"),
            (Operator::Has, "_", "has"),
        ];

        // Test suffix roundtrip conversions
        for (op, suffix, _) in &operators {
            assert_eq!(op.to_suffix(), *suffix);
            if !suffix.is_empty() {
                assert_eq!(
                    Operator::from_suffix(suffix.chars().next().expect("emptiness checked above")),
                    Some(*op)
                );
            }
        }

        // Test invalid suffix
        assert_eq!(Operator::from_suffix('x'), None);

        // Test string roundtrip conversions
        for (op, _, str_repr) in &operators {
            assert_eq!(op.as_str(), *str_repr);
            assert_eq!(Operator::from_str(str_repr), Some(*op));
        }

        // Test special cases
        assert_eq!(Operator::from_str("="), Some(Operator::Eq)); // Alternative for ==
        assert_eq!(Operator::from_str("foo"), None);

        // Test decode_filter returns Operator
        assert_eq!(decode_filter("property0"), ("property", Operator::Neq));
        assert_eq!(decode_filter("property1"), ("property", Operator::Lt));
        assert_eq!(decode_filter("property_"), ("property", Operator::Has));
        assert_eq!(decode_filter("property"), ("property", Operator::Eq));
        assert_eq!(decode_filter("p"), ("p", Operator::Eq));
    }

    #[test]
    fn strip_comments() {
        use super::strip_comments as s;

        assert_eq!(s(""), "");
        assert_eq!(s("// comment\nA"), "\nA");
        assert_eq!(s("A\n// comment\nB"), "A\n\nB");
        assert_eq!(s("A // comment\nB//comment"), "A \nB");
    }

    #[test]
    fn encode_filters() {
        use super::encode_filters as t;

        assert_eq!(t(""), "");
        assert_eq!(t(".a"), ".a");

        assert_eq!(t("(.a = 1)"), "(a   =1)");
        assert_eq!(t("(.a= 1)"), "(a  =1)");
        assert_eq!(t("(.a =1)"), "(a  =1)");
        assert_eq!(t("(.a=1)"), "(a =1)");

        assert_eq!(t("(.a == 1)"), "(a    =1)");
        assert_eq!(t("(.a== 1)"), "(a   =1)");
        assert_eq!(t("(.a ==1)"), "(a   =1)");
        assert_eq!(t("(.a==1)"), "(a  =1)");

        assert_eq!(t("(.a < 1)"), "(a1  =1)");
        assert_eq!(t("(.a< 1)"), "(a1 =1)");
        assert_eq!(t("(.a <1)"), "(a1 =1)");
        assert_eq!(t("(.a<1)"), "(a1=1)");

        assert_eq!(t("(.abc ~! 'regex')"), "(abc6   ='regex')");
        assert_eq!(t("(.abc~! 'regex')"), "(abc6  ='regex')");
        assert_eq!(t("(.abc ~!'regex')"), "(abc6  ='regex')");
        assert_eq!(t("(.abc~!'regex')"), "(abc6 ='regex')");

        assert_eq!(t("(.a != 1)"), "(a0   =1)");
        assert_eq!(t("(.a < 1)"), "(a1  =1)");
        assert_eq!(t("(.a <= 1)"), "(a2   =1)");
        assert_eq!(t("(.a > 1)"), "(a3  =1)");
        assert_eq!(t("(.a >= 1)"), "(a4   =1)");
        assert_eq!(t("(.a ~= 1)"), "(a5   =1)");
        assert_eq!(t("(.a ~! 1)"), "(a6   =1)");
        assert_eq!(t("(.a ^= 1)"), "(a7   =1)");
        assert_eq!(t("(.a $= 1)"), "(a8   =1)");
        assert_eq!(t("(.a in 1)"), "(a9   =1)");
        assert_eq!(t("(.a has 1)"), "(a_    =1)");

        assert_eq!(
            t("(.a != 1, .b < 1,.c has 1)"),
            "(a0   =1, b1  =1,c_    =1)"
        );

        assert_eq!(t("(above)"), "(above)");
        assert_eq!(t("(below, .a != 1)"), "(below, a0   =1)");

        assert_eq!(t("(* == 1)"), "(_C  =1)");
        assert_eq!(t("(* <  1)"), "(_C1 =1)");
        assert_eq!(t("(* > 1)"), "(_C3=1)");
        assert_eq!(t("(*>=1)"), "(_C4=1)");
    }
}
