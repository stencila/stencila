use std::path::Path;

use parser_treesitter::{
    common::{
        defaults::Defaults,
        eyre::Result,
        indexmap::IndexMap,
        itertools::Itertools,
        once_cell::sync::Lazy,
        regex::{Captures, Regex},
        tracing,
    },
    formats::Format,
    graph_triples::{relations, resources, Resource, ResourceInfo},
    resource_info, Parser, ParserTrait, TreesitterParser,
};
use stencila_schema::{
    BooleanValidator, Date, DateTime, DateTimeValidator, DateValidator, DurationValidator,
    EnumValidator, IntegerValidator, Node, Number, NumberValidator, Parameter, StringValidator,
    Time, TimeValidator, TimestampValidator, ValidatorTypes,
};

/// A parser for SQL
pub struct SqlParser;

impl ParserTrait for SqlParser {
    fn spec() -> Parser {
        Parser {
            language: Format::SQL,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        const QUERY: &str = include_str!("query.scm");
        static PARSER: Lazy<TreesitterParser> =
            Lazy::new(|| TreesitterParser::new(tree_sitter_sql::language(), QUERY));

        // Replace named bindings e.g. `$par_a` with numeric bindings e.g. `$1` so that
        // they are recognized by `tree-sitter-sql`.
        let mut bindings = Vec::new();
        static BINDING_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\$([a-zA-Z_][a-zA-Z_0-9]*)").expect("Unable to create regex")
        });
        let sql = BINDING_REGEX.replace_all(code, |captures: &Captures| {
            let name = captures[1].to_string();
            bindings.push(name);
            ["$", &bindings.len().to_string()].concat()
        });

        let code = sql.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, captures)| {
                let relation = match pattern {
                    1 => relations::assigns,
                    2 => relations::declares,
                    3 => relations::uses,
                    4 => relations::alters,
                    5 => relations::uses,
                    _ => return None,
                }(captures[0].range);

                let name = match pattern {
                    2 => [&captures[0].text, ".", &captures[1].text].concat(),
                    5 => match captures[0].text[1..].parse::<usize>() {
                        Ok(index) => match bindings.get(index - 1) {
                            Some(name) => name.to_string(),
                            None => return None,
                        },
                        Err(error) => {
                            tracing::error!(
                                "Unexpectedly unable to parse binding as integer index: {}",
                                error
                            );
                            return None;
                        }
                    },
                    _ => captures[0].text.to_string(),
                };

                let kind = match pattern {
                    2 => "DatatableColumn",
                    5 => "",
                    _ => "Datatable",
                };

                Some((relation, resources::symbol(path, &name, kind)))
            })
            .collect();

        let resource_info = resource_info(
            resource,
            path,
            Self::spec().language,
            code,
            &tree,
            &["comment"],
            matches,
            0,
            relations,
        );
        Ok(resource_info)
    }
}

impl SqlParser {
    /// Derive a set of [`Parameter`]s from a SQL `CREATE TABLE` statement
    pub fn derive_parameters(sql: &str) -> Vec<Parameter> {
        const DERIVE: &str = include_str!("derive.scm");
        static PARSER: Lazy<TreesitterParser> =
            Lazy::new(|| TreesitterParser::new(tree_sitter_sql::language(), DERIVE));

        // Parse and query the code
        let code = sql.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        // Create info on columns from captures
        let mut columns = IndexMap::new();
        'matches: for (_pattern, captures) in matches {
            let mut column = SqlColumn::default();
            let mut check = SqlCheck::default();
            for capture in captures {
                match capture.name.as_str() {
                    "name" => {
                        let name = capture.text;
                        column = if let Some(column) = columns.remove(&name) {
                            column
                        } else {
                            SqlColumn {
                                column_name: name.clone(),
                                ..Default::default()
                            }
                        }
                    }
                    "type" => {
                        column.data_type = capture.text;
                    }
                    "nullable" => {
                        if capture.text.to_uppercase().contains("NOT") {
                            column.is_nullable = false;
                        }
                    }
                    "default" => {
                        column.column_default = Some(capture.text);
                    }
                    "checks" => {
                        if !capture.text.to_uppercase().contains(" AND ") {
                            continue 'matches;
                        }
                    }
                    "check.op" => {
                        static OPERATOR_REGEX: Lazy<Regex> = Lazy::new(|| {
                            Regex::new(" (<|<=|>|>=) ").expect("Unable to create regex")
                        });
                        if let Some(captures) = OPERATOR_REGEX.captures(&capture.text) {
                            check.operator = captures[1].to_string();
                        }
                    }
                    "check.in" => {
                        check.operator = "in".to_string();
                    }
                    "check.identifier" => {
                        if capture.text != column.column_name {
                            // This check is for a different column so reset the operator
                            // so that it is not included
                            check.operator = String::new();
                        }
                    }
                    "check.call" => {
                        if let Some(function) = capture
                            .text
                            .strip_suffix(&["(", &column.column_name, ")"].concat())
                        {
                            check.function = function.to_owned();
                        } else {
                            // This check is for a different column so do as above
                            check.operator = String::new();
                        }
                    }
                    "check.number" => {
                        check.number = capture.text;
                    }
                    "check.string" => {
                        check.string = capture.text;
                    }
                    "check.tuple" => {
                        let tuple = capture.text;
                        let tuple = tuple.strip_prefix('(').unwrap_or(&tuple);
                        let tuple = tuple.strip_suffix(')').unwrap_or(tuple);

                        static ITEMS_REGEX: Lazy<Regex> =
                            Lazy::new(|| Regex::new(r"'\s*,\s*'").expect("Unable to create regex"));
                        let items = ITEMS_REGEX
                            .split(tuple)
                            .map(|item| item.strip_prefix('\'').unwrap_or(item))
                            .map(|item| item.strip_suffix('\'').unwrap_or(item))
                            .map(String::from)
                            .collect_vec();

                        if !items.is_empty() {
                            check.tuple = items;
                        }
                    }
                    _ => {}
                }
            }
            if !check.operator.is_empty() {
                column.checks.push(check);
            }
            columns.insert(column.column_name.clone(), column);
        }

        // Convert each column definition into a parameter
        columns
            .into_values()
            .map(|column| column.derive_parameter())
            .collect_vec()
    }
}

/// A column in a SQL database column
///
/// The field names used below correspond to those in the
/// `information_schema.columns` view of the SQL standard
/// See for example https://duckdb.org/docs/sql/information_schema#columns
#[derive(Debug, Defaults)]
pub struct SqlColumn {
    column_name: String,

    data_type: String,

    #[def = "true"]
    is_nullable: bool,

    column_default: Option<String>,

    checks: Vec<SqlCheck>,
}

#[derive(Debug, Defaults)]
struct SqlCheck {
    function: String,

    operator: String,

    number: String,

    string: String,

    tuple: Vec<String>,
}

impl SqlColumn {
    /// Derive a [`Parameter`] from the properties of a SQL table column
    pub fn derive_parameter(self) -> Parameter {
        let data_type = self.data_type.to_uppercase();
        let mut validator = match data_type.as_ref() {
            "BOOLEAN" | "BOOL" | "LOGICAL" => {
                Some(ValidatorTypes::BooleanValidator(BooleanValidator::default()))
            }
            "INTEGER" | "INT" | "TINYINT" | "SMALLINT" | "MEDIUMINT" | "BIGINT" | "INT1"
            | "INT2" | "INT4" | "INT8" | "SHORT" | "LONG" | "SMALLSERIAL" | "SERIAL"
            | "BIGSERIAL" => Some(ValidatorTypes::IntegerValidator(IntegerValidator::default())),
            "UNSIGNED INTEGER" | "UINT" | "UTINYINT" | "USMALLINT" | "UMEDIUMINT" | "UBIGINT"
            | "UINT1" | "UINT2" | "UINT4" | "UINT8" => {
                Some(ValidatorTypes::IntegerValidator(IntegerValidator {
                    minimum: Some(Number(0f64)),
                    ..Default::default()
                }))
            }
            "REAL" | "DOUBLE" | "DOUBLE PRECISION" | "FLOAT" | "FLOAT4" | "FLOAT8" | "NUMERIC"
            | "DECIMAL" => Some(ValidatorTypes::NumberValidator(NumberValidator::default())),
            "TEXT" | "CHARACTER" | "CHAR" | "BPCHAR" | "STRING" => {
                Some(ValidatorTypes::StringValidator(StringValidator::default()))
            }
            "DATE" => Some(ValidatorTypes::DateValidator(DateValidator::default())),
            "TIME" => Some(ValidatorTypes::TimeValidator(TimeValidator::default())),
            "DATETIME" => Some(ValidatorTypes::DateTimeValidator(
                DateTimeValidator::default(),
            )),
            // TODO: deal with more complicated timestamp and interval types; see https://www.postgresql.org/docs/current/datatype-datetime.html
            "TIMESTAMP" => Some(ValidatorTypes::TimestampValidator(
                TimestampValidator::default(),
            )),
            "INTERVAL" => Some(ValidatorTypes::DurationValidator(
                DurationValidator::default(),
            )),
            _ => {
                static VARCHAR_REGEX: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(r"(VARCHAR|CHARACTER VARYING|CHARACTER|CHAR)\(\d+\)")
                        .expect("Unable to create regex")
                });

                if data_type.starts_with("DECIMAL") {
                    Some(ValidatorTypes::NumberValidator(NumberValidator::default()))
                } else if let Some(captures) = VARCHAR_REGEX.captures(data_type.as_ref()) {
                    let max_length = if let Ok(length) = captures[2].to_string().parse::<u32>() {
                        Some(length)
                    } else {
                        None
                    };
                    Some(ValidatorTypes::StringValidator(StringValidator {
                        max_length,
                        ..Default::default()
                    }))
                } else {
                    None
                }
            }
        };

        if let Some(ValidatorTypes::IntegerValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.is_empty() && !check.number.is_empty() {
                    let number = check.number.parse().ok().map(Number);
                    if number.is_some() {
                        if check.operator == ">=" {
                            validator.minimum = number;
                        } else if check.operator == ">" {
                            validator.exclusive_minimum = number;
                        } else if check.operator == "<=" {
                            validator.maximum = number;
                        } else if check.operator == "<" {
                            validator.exclusive_maximum = number;
                        }
                    }
                }
            }
        } else if let Some(ValidatorTypes::NumberValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.is_empty() && !check.number.is_empty() {
                    let number = check.number.parse().ok().map(Number);
                    if number.is_some() {
                        if check.operator == ">=" {
                            validator.minimum = number;
                        } else if check.operator == ">" {
                            validator.exclusive_minimum = number;
                        } else if check.operator == "<=" {
                            validator.maximum = number;
                        } else if check.operator == "<" {
                            validator.exclusive_maximum = number;
                        }
                    }
                }
            }
        } else if let Some(ValidatorTypes::StringValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.to_lowercase() == "length" && !check.number.is_empty() {
                    if let Ok(length) = check.number.parse::<u32>() {
                        if check.operator == ">=" {
                            validator.min_length = Some(length);
                        } else if check.operator == ">" {
                            validator.min_length = Some(length + 1);
                        } else if check.operator == "<=" {
                            validator.max_length = Some(length);
                        } else if check.operator == "<" {
                            validator.max_length = Some(length - 1);
                        }
                    }
                }
            }
        } else if let Some(ValidatorTypes::DateValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.is_empty() && !check.string.is_empty() {
                    if check.operator == ">=" || check.operator == ">" {
                        validator.minimum = Some(Date::from(check.string.clone()));
                    } else if check.operator == "<=" || check.operator == "<" {
                        validator.maximum = Some(Date::from(check.string.clone()));
                    }
                }
            }
        } else if let Some(ValidatorTypes::TimeValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.is_empty() && !check.string.is_empty() {
                    if check.operator == ">=" || check.operator == ">" {
                        validator.minimum = Some(Time::from(check.string.clone()));
                    } else if check.operator == "<=" || check.operator == "<" {
                        validator.maximum = Some(Time::from(check.string.clone()));
                    }
                }
            }
        } else if let Some(ValidatorTypes::DateTimeValidator(validator)) = &mut validator {
            for check in &self.checks {
                if check.function.is_empty() && !check.string.is_empty() {
                    if check.operator == ">=" || check.operator == ">" {
                        validator.minimum = Some(DateTime::from(check.string.clone()));
                    } else if check.operator == "<=" || check.operator == "<" {
                        validator.maximum = Some(DateTime::from(check.string.clone()));
                    }
                }
            }
        }

        // If there is a `CHECK(col IN (...))` clause then make the validator and enum validator
        if matches!(validator, None | Some(ValidatorTypes::StringValidator(..))) {
            for check in self.checks {
                if check.function.is_empty() && !check.tuple.is_empty() {
                    validator = Some(ValidatorTypes::EnumValidator(EnumValidator {
                        values: check
                            .tuple
                            .iter()
                            .map(|item| Node::String(item.to_owned()))
                            .collect_vec(),
                        ..Default::default()
                    }));
                    break;
                }
            }
        }

        let default = if let Some(default) = self.column_default {
            match validator {
                Some(ValidatorTypes::BooleanValidator(..)) => {
                    default.parse().ok().map(Node::Boolean)
                }
                Some(ValidatorTypes::IntegerValidator(..)) => {
                    default.parse().ok().map(Node::Integer)
                }
                Some(ValidatorTypes::NumberValidator(..)) => default.parse().ok().map(Node::Number),
                Some(ValidatorTypes::StringValidator(..)) => Some(Node::String(default)),
                Some(ValidatorTypes::DateValidator(..)) => Some(Node::Date(Date::from(default))),
                Some(ValidatorTypes::TimeValidator(..)) => Some(Node::Time(Time::from(default))),
                Some(ValidatorTypes::DateTimeValidator(..)) => {
                    Some(Node::DateTime(DateTime::from(default)))
                }
                _ => Some(Node::String(default)),
            }
        } else {
            None
        };

        Parameter {
            name: self.column_name.to_owned(),
            validator: validator.map(Box::new),
            default: default.map(Box::new),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    use super::*;

    #[test]
    fn parse_sql_fragments() {
        snapshot_fixtures("fragments/sql/*.sql", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(path, "", "SoftwareSourceCode", Format::SQL);
            let resource_info = SqlParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }

    /// Regression test for when a numeric binding is in the SQL code
    #[test]
    fn do_not_panic_on_numeric_bindings() -> Result<()> {
        let code = "SELECT * FROM table_1 WHERE col_1 = $1 OR col_1 = ?1";
        let path = PathBuf::new();
        let resource = resources::code(&path, "", "SoftwareSourceCode", Format::SQL);
        SqlParser::parse(resource, &path, code)?;
        Ok(())
    }

    #[test]
    fn derive_parameters() -> Result<()> {
        let sql = std::fs::read_to_string(fixtures().join("fragments/sql/create-table.sql"))
            .expect("Unable to read");
        let parameters = SqlParser::derive_parameters(&sql);
        assert_json_snapshot!(parameters);
        Ok(())
    }
}
