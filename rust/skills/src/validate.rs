use regex::Regex;
use thiserror::Error;

use stencila_schema::Skill;

/// Validation errors for a skill per the Agent Skills Specification
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("name must not be empty")]
    NameEmpty,

    #[error("name must be at most 64 characters, got {0}")]
    NameTooLong(usize),

    #[error("name may only contain lowercase alphanumeric characters and hyphens")]
    NameInvalidChars,

    #[error("name must not start with a hyphen")]
    NameLeadingHyphen,

    #[error("name must not end with a hyphen")]
    NameTrailingHyphen,

    #[error("name must not contain consecutive hyphens")]
    NameConsecutiveHyphens,

    #[error("name `{name}` does not match directory name `{dir_name}`")]
    NameDirMismatch { name: String, dir_name: String },

    #[error("description must not be empty")]
    DescriptionEmpty,

    #[error("description appears to be a placeholder")]
    DescriptionPlaceholder,

    #[error("description must be at most 1024 characters, got {0}")]
    DescriptionTooLong(usize),

    #[error("compatibility must not be empty if provided")]
    CompatibilityEmpty,

    #[error("compatibility must be at most 500 characters, got {0}")]
    CompatibilityTooLong(usize),
}

/// Validate a skill name per the Agent Skills Specification
///
/// Rules:
/// - Must be 1-64 characters
/// - Only lowercase alphanumeric characters and hyphens
/// - Must not start or end with a hyphen
/// - Must not contain consecutive hyphens
pub fn validate_name(name: &str) -> Vec<ValidationError> {
    // Use `std::sync::LazyLock` when stabilized in our MSRV; for now just compile each time
    // (validation is not a hot path)
    let valid_chars =
        Regex::new(r"^[a-z0-9\-]+$").expect("validate_name: invalid regex should not happen");

    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push(ValidationError::NameEmpty);
        return errors;
    }

    if name.len() > 64 {
        errors.push(ValidationError::NameTooLong(name.len()));
    }

    if !valid_chars.is_match(name) {
        errors.push(ValidationError::NameInvalidChars);
    }

    if name.starts_with('-') {
        errors.push(ValidationError::NameLeadingHyphen);
    }

    if name.ends_with('-') {
        errors.push(ValidationError::NameTrailingHyphen);
    }

    if name.contains("--") {
        errors.push(ValidationError::NameConsecutiveHyphens);
    }

    errors
}

/// Validate a skill against the Agent Skills Specification
///
/// Optionally checks that the name matches the parent directory name.
pub fn validate_skill(skill: &Skill, dir_name: Option<&str>) -> Vec<ValidationError> {
    let mut errors = validate_name(&skill.name);

    if let Some(dir_name) = dir_name
        && skill.name != dir_name
    {
        errors.push(ValidationError::NameDirMismatch {
            name: skill.name.clone(),
            dir_name: dir_name.to_string(),
        });
    }

    if skill.description.is_empty() {
        errors.push(ValidationError::DescriptionEmpty);
    } else if skill.description.trim().eq_ignore_ascii_case("todo") {
        errors.push(ValidationError::DescriptionPlaceholder);
    } else if skill.description.len() > 1024 {
        errors.push(ValidationError::DescriptionTooLong(skill.description.len()));
    }

    if let Some(compat) = &skill.compatibility {
        if compat.is_empty() {
            errors.push(ValidationError::CompatibilityEmpty);
        } else if compat.len() > 500 {
            errors.push(ValidationError::CompatibilityTooLong(compat.len()));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        assert!(validate_name("data-analysis").is_empty());
        assert!(validate_name("data-analysis").is_empty());
        assert!(validate_name("code-review").is_empty());
        assert!(validate_name("a").is_empty());
        assert!(validate_name("abc123").is_empty());
        Ok(())
    }

    #[test]
    fn invalid_names() -> eyre::Result<()> {
        assert!(validate_name("").contains(&ValidationError::NameEmpty));

        assert!(validate_name("Data-Analysis").contains(&ValidationError::NameInvalidChars));

        assert!(validate_name("-data").contains(&ValidationError::NameLeadingHyphen));

        assert!(validate_name("data-").contains(&ValidationError::NameTrailingHyphen));

        assert!(validate_name("data--analysis").contains(&ValidationError::NameConsecutiveHyphens));

        let long_name = "a".repeat(65);
        assert!(validate_name(&long_name).contains(&ValidationError::NameTooLong(65)));

        Ok(())
    }

    #[test]
    fn validate_skill_checks() -> eyre::Result<()> {
        use stencila_schema::Skill;

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test skill".into(),
            ..Default::default()
        };

        assert!(validate_skill(&skill, None).is_empty());
        assert!(validate_skill(&skill, Some("test-skill")).is_empty());

        let errors = validate_skill(&skill, Some("other-dir"));
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::NameDirMismatch { .. }))
        );

        let empty_desc = Skill {
            name: "test".into(),
            description: String::new(),
            ..Default::default()
        };
        assert!(validate_skill(&empty_desc, None).contains(&ValidationError::DescriptionEmpty));

        let long_desc = Skill {
            name: "test".into(),
            description: "x".repeat(1025),
            ..Default::default()
        };
        assert!(
            validate_skill(&long_desc, None)
                .iter()
                .any(|e| matches!(e, ValidationError::DescriptionTooLong(1025)))
        );

        let todo_desc = Skill {
            name: "test".into(),
            description: "TODO".into(),
            ..Default::default()
        };
        assert!(
            validate_skill(&todo_desc, None).contains(&ValidationError::DescriptionPlaceholder)
        );

        let todo_desc_lower = Skill {
            name: "test".into(),
            description: " todo ".into(),
            ..Default::default()
        };
        assert!(
            validate_skill(&todo_desc_lower, None)
                .contains(&ValidationError::DescriptionPlaceholder)
        );

        let empty_compat = Skill {
            name: "test".into(),
            description: "A test".into(),
            compatibility: Some(String::new()),
            ..Default::default()
        };
        assert!(validate_skill(&empty_compat, None).contains(&ValidationError::CompatibilityEmpty));

        let long_compat = Skill {
            name: "test".into(),
            description: "A test".into(),
            compatibility: Some("x".repeat(501)),
            ..Default::default()
        };
        assert!(
            validate_skill(&long_compat, None)
                .iter()
                .any(|e| matches!(e, ValidationError::CompatibilityTooLong(501)))
        );

        Ok(())
    }
}
