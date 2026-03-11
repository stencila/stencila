use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Equals { key: String, value: String },
    NotEquals { key: String, value: String },
}

impl Condition {
    /// Parse a condition string of the form `"key == value"` or `"key != value"`.
    ///
    /// Both the key and value are trimmed of surrounding whitespace. The value
    /// portion is everything after the operator, so it may contain spaces
    /// (e.g., `"role == senior engineer"`).
    pub fn parse(s: &str) -> Result<Self, String> {
        // Try `!=` first so we don't accidentally match `=` inside `!=`.
        let (key, value, is_equals) = if let Some((k, v)) = s.split_once("!=") {
            (k.trim(), v.trim(), false)
        } else if let Some((k, v)) = s.split_once("==") {
            (k.trim(), v.trim(), true)
        } else {
            return Err(format!(
                "condition missing operator: expected `==` or `!=` in \"{s}\""
            ));
        };

        if key.is_empty() {
            return Err(format!("condition has empty key in \"{s}\""));
        }
        if value.is_empty() {
            return Err(format!("condition has empty value in \"{s}\""));
        }

        let key = key.to_string();
        let value = value.to_string();

        if is_equals {
            Ok(Condition::Equals { key, value })
        } else {
            Ok(Condition::NotEquals { key, value })
        }
    }

    /// Evaluate this condition against a map of answers.
    ///
    /// - `Equals`: true when the key exists **and** the values match
    ///   (case-insensitive ASCII comparison on trimmed strings).
    /// - `NotEquals`: true when the key is missing **or** the values do not
    ///   match (case-insensitive ASCII comparison on trimmed strings).
    pub fn evaluate(&self, answers: &IndexMap<String, String>) -> bool {
        match self {
            Condition::Equals { key, value } => answers
                .get(key)
                .is_some_and(|ans| ans.trim().eq_ignore_ascii_case(value.trim())),
            Condition::NotEquals { key, value } => answers
                .get(key)
                .is_none_or(|ans| !ans.trim().eq_ignore_ascii_case(value.trim())),
        }
    }

    /// Return the key referenced by this condition.
    pub fn referenced_key(&self) -> &str {
        match self {
            Condition::Equals { key, .. } | Condition::NotEquals { key, .. } => key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Parsing – valid conditions
    // -----------------------------------------------------------------------

    #[test]
    fn parse_equals() -> Result<(), String> {
        let c = Condition::parse("role == admin")?;
        assert_eq!(
            c,
            Condition::Equals {
                key: "role".into(),
                value: "admin".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn parse_not_equals() -> Result<(), String> {
        let c = Condition::parse("env != production")?;
        assert_eq!(
            c,
            Condition::NotEquals {
                key: "env".into(),
                value: "production".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn parse_trims_whitespace() -> Result<(), String> {
        let c = Condition::parse("  key   ==   val  ")?;
        assert_eq!(
            c,
            Condition::Equals {
                key: "key".into(),
                value: "val".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn parse_value_with_spaces() -> Result<(), String> {
        let c = Condition::parse("role == senior engineer")?;
        assert_eq!(
            c,
            Condition::Equals {
                key: "role".into(),
                value: "senior engineer".into(),
            }
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Parsing – errors
    // -----------------------------------------------------------------------

    #[test]
    fn parse_error_no_operator() {
        let err = Condition::parse("no operator here").expect_err("should fail");
        assert!(err.contains("missing operator"), "got: {err}");
    }

    #[test]
    fn parse_error_empty_key() {
        let err = Condition::parse(" == value").expect_err("should fail");
        assert!(err.contains("empty key"), "got: {err}");
    }

    #[test]
    fn parse_error_empty_value() {
        let err = Condition::parse("key ==  ").expect_err("should fail");
        assert!(err.contains("empty value"), "got: {err}");
    }

    #[test]
    fn parse_error_empty_key_not_equals() {
        let err = Condition::parse(" != value").expect_err("should fail");
        assert!(err.contains("empty key"), "got: {err}");
    }

    #[test]
    fn parse_error_empty_value_not_equals() {
        let err = Condition::parse("key !=  ").expect_err("should fail");
        assert!(err.contains("empty value"), "got: {err}");
    }

    // -----------------------------------------------------------------------
    // Evaluate – Equals
    // -----------------------------------------------------------------------

    fn answers(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn evaluate_equals_match() {
        let c = Condition::Equals {
            key: "role".into(),
            value: "admin".into(),
        };
        assert!(c.evaluate(&answers(&[("role", "admin")])));
    }

    #[test]
    fn evaluate_equals_no_match() {
        let c = Condition::Equals {
            key: "role".into(),
            value: "admin".into(),
        };
        assert!(!c.evaluate(&answers(&[("role", "user")])));
    }

    #[test]
    fn evaluate_equals_missing_key() {
        let c = Condition::Equals {
            key: "role".into(),
            value: "admin".into(),
        };
        assert!(!c.evaluate(&answers(&[])));
    }

    // -----------------------------------------------------------------------
    // Evaluate – NotEquals
    // -----------------------------------------------------------------------

    #[test]
    fn evaluate_not_equals_match_returns_false() {
        let c = Condition::NotEquals {
            key: "env".into(),
            value: "prod".into(),
        };
        assert!(!c.evaluate(&answers(&[("env", "prod")])));
    }

    #[test]
    fn evaluate_not_equals_no_match_returns_true() {
        let c = Condition::NotEquals {
            key: "env".into(),
            value: "prod".into(),
        };
        assert!(c.evaluate(&answers(&[("env", "staging")])));
    }

    #[test]
    fn evaluate_not_equals_missing_key_returns_true() {
        let c = Condition::NotEquals {
            key: "env".into(),
            value: "prod".into(),
        };
        assert!(c.evaluate(&answers(&[])));
    }

    // -----------------------------------------------------------------------
    // Case-insensitive comparison
    // -----------------------------------------------------------------------

    #[test]
    fn evaluate_case_insensitive_equals() {
        let c = Condition::Equals {
            key: "lang".into(),
            value: "Rust".into(),
        };
        assert!(c.evaluate(&answers(&[("lang", "rust")])));
        assert!(c.evaluate(&answers(&[("lang", "RUST")])));
        assert!(c.evaluate(&answers(&[("lang", "rUsT")])));
    }

    #[test]
    fn evaluate_case_insensitive_not_equals() {
        let c = Condition::NotEquals {
            key: "lang".into(),
            value: "Rust".into(),
        };
        assert!(!c.evaluate(&answers(&[("lang", "rust")])));
        assert!(!c.evaluate(&answers(&[("lang", "RUST")])));
        assert!(c.evaluate(&answers(&[("lang", "Python")])));
    }

    // -----------------------------------------------------------------------
    // referenced_key
    // -----------------------------------------------------------------------

    #[test]
    fn referenced_key_equals() {
        let c = Condition::Equals {
            key: "role".into(),
            value: "admin".into(),
        };
        assert_eq!(c.referenced_key(), "role");
    }

    #[test]
    fn referenced_key_not_equals() {
        let c = Condition::NotEquals {
            key: "env".into(),
            value: "prod".into(),
        };
        assert_eq!(c.referenced_key(), "env");
    }
}
