//! Database packs: `database.postgresql`, `database.mysql`, `database.sqlite`.

use std::sync::LazyLock;

use regex::Regex;

use super::{Confidence, Pack, PatternRule, destructive_pattern};

/// Compiled regex for extracting SQL after `DELETE FROM <table>`.
static DELETE_FROM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bDELETE\s+FROM\s+\w+").expect("delete_from regex should compile")
});

/// Validator for `delete_no_where`: fires when `DELETE FROM <table>` is NOT
/// followed by a `WHERE` clause.
fn delete_no_where_validator(cmd: &str) -> bool {
    if let Some(m) = DELETE_FROM_RE.find(cmd) {
        let remainder = &cmd[m.end()..];
        let remainder_upper = remainder.to_ascii_uppercase();
        !remainder_upper.contains("WHERE")
    } else {
        false
    }
}

// Shared SQL patterns (case-insensitive).

pub static POSTGRESQL_PACK: Pack = Pack {
    id: "database.postgresql",
    name: "PostgreSQL",
    description: "Guards against destructive PostgreSQL operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "drop_database",
            r"(?i)\bDROP\s+DATABASE\b",
            "Permanently destroys the entire database",
            "Use `pg_dump` to backup first",
            Confidence::High
        ),
        destructive_pattern!(
            "drop_table",
            r"(?i)\bDROP\s+TABLE\b",
            "Permanently destroys a table and all its data",
            "Use `pg_dump -t <table>` to backup first; use a transaction with `BEGIN`/`ROLLBACK` for safety",
            Confidence::High
        ),
        destructive_pattern!(
            "truncate",
            r"(?i)\bTRUNCATE\s+(?:TABLE\s+)?\w",
            "Removes all rows without logging individual deletions",
            "Use `DELETE FROM` with a `WHERE` clause for selective deletion",
            Confidence::Medium
        ),
        destructive_pattern!(
            "delete_no_where",
            r"(?i)\bDELETE\s+FROM\b",
            delete_no_where_validator,
            "Deletes all rows from a table",
            "Add a `WHERE` clause to limit deletion scope. If your query already has a WHERE clause on a separate line, combine them onto one line (e.g., `DELETE FROM users WHERE active = false`)",
            Confidence::Medium
        ),
    ],
};

pub static MYSQL_PACK: Pack = Pack {
    id: "database.mysql",
    name: "MySQL",
    description: "Guards against destructive MySQL operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "drop_database",
            r"(?i)\bDROP\s+(?:DATABASE|SCHEMA)\b",
            "Permanently destroys the entire database",
            "Use `mysqldump` to backup first",
            Confidence::High
        ),
        destructive_pattern!(
            "drop_table",
            r"(?i)\bDROP\s+TABLE\b",
            "Permanently destroys a table and all its data",
            "Backup the table first with `mysqldump`",
            Confidence::High
        ),
        destructive_pattern!(
            "truncate",
            r"(?i)\bTRUNCATE\s+(?:TABLE\s+)?\w",
            "Removes all rows without logging",
            "Use `DELETE FROM` with a `WHERE` clause",
            Confidence::Medium
        ),
        destructive_pattern!(
            "delete_no_where",
            r"(?i)\bDELETE\s+FROM\b",
            delete_no_where_validator,
            "Deletes all rows from a table",
            "Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line",
            Confidence::Medium
        ),
    ],
};

pub static SQLITE_PACK: Pack = Pack {
    id: "database.sqlite",
    name: "SQLite",
    description: "Guards against destructive SQLite operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "drop_table",
            r"(?i)\bDROP\s+TABLE\b",
            "Permanently destroys a table",
            "Backup the database file first",
            Confidence::High
        ),
        destructive_pattern!(
            "delete_no_where",
            r"(?i)\bDELETE\s+FROM\b",
            delete_no_where_validator,
            "Deletes all rows from a table",
            "Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    #[test]
    fn drop_database_matches() {
        let re = Regex::new(rule_by_id(&POSTGRESQL_PACK, "drop_database").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("DROP DATABASE mydb"));
        assert!(re.is_match("drop database mydb"));
        assert!(!re.is_match("SELECT * FROM databases"));
    }

    #[test]
    fn drop_table_matches() {
        let re = Regex::new(rule_by_id(&POSTGRESQL_PACK, "drop_table").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("DROP TABLE users"));
        assert!(re.is_match("DROP TABLE IF EXISTS users"));
        assert!(re.is_match("drop table users"));
        assert!(!re.is_match("SELECT * FROM users"));
    }

    #[test]
    fn truncate_matches() {
        let re = Regex::new(rule_by_id(&POSTGRESQL_PACK, "truncate").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("TRUNCATE TABLE users"));
        assert!(re.is_match("TRUNCATE users"));
        assert!(!re.is_match("SELECT * FROM users"));
    }

    #[test]
    fn delete_no_where_matches() {
        // Pattern matches broadly
        let re = Regex::new(rule_by_id(&POSTGRESQL_PACK, "delete_no_where").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("DELETE FROM users"));
        assert!(re.is_match("DELETE FROM users WHERE id = 1"));

        // Validator narrows to only fire without WHERE
        assert!(delete_no_where_validator("DELETE FROM users"));
        assert!(delete_no_where_validator("DELETE FROM users;"));
        assert!(!delete_no_where_validator("DELETE FROM users WHERE id = 1"));
        assert!(!delete_no_where_validator(
            "delete from users where active = false"
        ));
        // Quoted SQL in shell commands
        assert!(delete_no_where_validator("psql -c 'DELETE FROM users'"));
        assert!(!delete_no_where_validator(
            "psql -c 'DELETE FROM users WHERE id = 1'"
        ));
        assert!(delete_no_where_validator(
            r#"sqlite3 db.sqlite "DELETE FROM users;""#
        ));
        assert!(!delete_no_where_validator(
            r#"sqlite3 db.sqlite "DELETE FROM users WHERE id = 1;""#
        ));
    }

    #[test]
    fn mysql_drop_schema() {
        let re = Regex::new(rule_by_id(&MYSQL_PACK, "drop_database").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("DROP SCHEMA mydb"));
        assert!(re.is_match("DROP DATABASE mydb"));
    }
}
