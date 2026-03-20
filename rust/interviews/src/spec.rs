use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smart_default::SmartDefault;

use crate::condition::Condition;
use crate::interviewer::{Answer, AnswerValue, Interview, Question, QuestionOption, QuestionType};

/// Declarative specification for an interview, typically deserialized from
/// YAML or JSON configuration (e.g., inside an `AGENT.md` frontmatter).
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InterviewSpec {
    /// Markdown content rendered before the questions as persistent interview
    /// context. Use this for any explanatory content that includes block-level
    /// formatting such as numbered lists, bullet lists, headings, or multiple
    /// paragraphs. The question text field only supports plain text and inline
    /// formatting, so longer structured content must go here.
    pub preamble: Option<String>,

    /// One or more questions to present to the user.
    pub questions: Vec<QuestionSpec>,
}

/// Specification for a single question within an [`InterviewSpec`].
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct QuestionSpec {
    /// The question text to present. Keep this concise (one or two plain-text
    /// sentences). If you need to provide longer context with lists, headings,
    /// or multiple paragraphs, put that content in the top-level preamble
    /// field instead.
    pub question: String,

    /// Short label displayed above the question.
    pub header: Option<String>,

    /// The type of question. Defaults to 'freeform'.
    #[serde(default)]
    pub r#type: QuestionTypeSpec,

    /// Choices for single-select or multi_select questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<OptionSpec>,

    /// Default answer used when the user skips or times out. For yes-no and
    /// confirm: 'yes' or 'no'. For freeform: the default text. For
    /// single-select: the label of the default option. For multi_select:
    /// comma-separated labels of default options (empty string for no
    /// defaults). Option labels must not contain commas.
    pub default: Option<String>,

    /// Context key under which this question's answer is stored. Required for
    /// answers that downstream nodes need to reference via `$KEY` expansion.
    pub store: Option<String>,

    /// End the interview immediately if the answer matches this value.
    ///
    /// For `yes-no` and `confirm` questions: `"yes"` or `"no"`.
    /// For `single-select` questions: an option label.
    /// Not supported for `freeform` or `multi_select` questions.
    ///
    /// When triggered, remaining questions are not presented and the
    /// interview completes successfully with the answers collected so far.
    #[serde(alias = "finish-if")]
    pub finish_if: Option<String>,

    /// Only present this question when the condition is true.
    ///
    /// Syntax: `"store_key == value"` or `"store_key != value"`, where
    /// `store_key` is the `store` name of an earlier question. Comparisons
    /// are case-insensitive. If the referenced key has not been answered
    /// (because its question was itself skipped), `==` evaluates to false
    /// and `!=` evaluates to true.
    #[serde(alias = "show-if")]
    pub show_if: Option<String>,
}

/// The type of question in a spec, using kebab-case naming with alias for other casing.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    SmartDefault,
    Deserialize,
    Serialize,
    JsonSchema,
    strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum QuestionTypeSpec {
    #[serde(alias = "yes_no")]
    YesNo,

    #[serde(alias = "confirmation")]
    Confirm,

    #[serde(
        alias = "single_select",
        alias = "multi_choice",
        alias = "multi-choice",
        alias = "multiple_choice",
        alias = "multiple-choice"
    )]
    SingleSelect,

    #[serde(
        alias = "multi_select",
        alias = "multiple_select",
        alias = "multiple-select"
    )]
    MultiSelect,

    #[default]
    #[serde(alias = "free", alias = "free-form", alias = "free-text")]
    Freeform,
}

/// A selectable option for multiple-choice / multi-select spec questions.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OptionSpec {
    /// Display text for the option.
    pub label: String,

    /// Explanatory text shown alongside the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl InterviewSpec {
    /// Parse an `InterviewSpec` from a string, trying YAML first then JSON.
    ///
    /// # Errors
    ///
    /// Returns a human-readable error message if neither format succeeds.
    pub fn parse(spec: &str) -> Result<Self, String> {
        serde_yaml::from_str(spec)
            .or_else(|yaml_err| serde_json::from_str(spec).map_err(|_| yaml_err))
            .map_err(|e| format!("failed to parse interview spec: {e}"))
    }

    /// Returns `true` if any question uses conditional features
    /// (`finish-if` or `show_if`), meaning the interview must be
    /// conducted progressively (one question at a time) rather than
    /// as a flat batch.
    #[must_use]
    pub fn is_conditional(&self) -> bool {
        self.questions
            .iter()
            .any(|q| q.finish_if.is_some() || q.show_if.is_some())
    }

    /// Validate the semantic correctness of this spec.
    ///
    /// Checks that:
    /// - `show_if` conditions parse correctly
    /// - `show_if` references only `store` keys defined by earlier questions
    /// - `finish-if` is only used with compatible question types
    ///   (`yes-no`, `confirm`, `single-select`)
    /// - There are no duplicate `store` keys
    ///
    /// Returns `Ok(())` if valid, or `Err` with a list of all problems found.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut seen_stores = HashSet::new();
        let mut available_stores = HashSet::<&str>::new();

        for (i, q) in self.questions.iter().enumerate() {
            let label = q.header.as_deref().unwrap_or(q.question.as_str());

            // Check for duplicate store keys
            if let Some(ref store) = q.store
                && !seen_stores.insert(store.as_str())
            {
                errors.push(format!(
                    "question {i} ({label}): duplicate store key '{store}'"
                ));
            }

            // Validate show_if
            if let Some(ref show_if) = q.show_if {
                match Condition::parse(show_if) {
                    Ok(cond) => {
                        let key = cond.referenced_key();
                        if !available_stores.contains(key) {
                            errors.push(format!(
                                "question {i} ({label}): show-if references \
                                 '{key}' which is not a store key from an \
                                 earlier question"
                            ));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("question {i} ({label}): invalid show-if: {e}"));
                    }
                }
            }

            // Validate finish_if
            if let Some(ref finish_if) = q.finish_if {
                match q.r#type {
                    QuestionTypeSpec::Freeform => {
                        errors.push(format!(
                            "question {i} ({label}): finish_if is not \
                             supported for freeform questions"
                        ));
                    }
                    QuestionTypeSpec::MultiSelect => {
                        errors.push(format!(
                            "question {i} ({label}): finish_if is not \
                             supported for multi_select questions"
                        ));
                    }
                    QuestionTypeSpec::YesNo | QuestionTypeSpec::Confirm => {
                        let lower = finish_if.trim().to_lowercase();
                        if !matches!(lower.as_str(), "yes" | "no") {
                            errors.push(format!(
                                "question {i} ({label}): finish-if for \
                                 {} must be 'yes' or 'no', got '{finish_if}'",
                                q.r#type
                            ));
                        }
                    }
                    QuestionTypeSpec::SingleSelect => {
                        let trimmed = finish_if.trim();
                        if !q
                            .options
                            .iter()
                            .any(|o| o.label.trim().eq_ignore_ascii_case(trimmed))
                        {
                            errors.push(format!(
                                "question {i} ({label}): finish-if value \
                                 '{finish_if}' does not match any option label"
                            ));
                        }
                    }
                }
            }

            // Add this question's store to available set for subsequent
            // questions' show-if references.
            if let Some(ref store) = q.store {
                available_stores.insert(store.as_str());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert this spec into a runtime [`Interview`].
    ///
    /// `stage` is the originating pipeline stage name (e.g., a node ID).
    pub fn to_interview(&self, stage: &str) -> Result<Interview, String> {
        let questions: Vec<Question> = self
            .questions
            .iter()
            .map(|qs| qs.to_question())
            .collect::<Result<Vec<_>, _>>()?;

        let mut interview = match <[Question; 1]>::try_from(questions) {
            Ok([q]) => Interview::single(q, stage),
            Err(questions) => Interview::batch(questions, stage),
        };

        interview.preamble = self.preamble.clone();
        Ok(interview)
    }
}

/// Assign an option key for a zero-based index: A..Z then "27", "28", …
fn option_key(index: usize) -> String {
    if index < 26 {
        String::from((b'A' + index as u8) as char)
    } else {
        (index + 1).to_string()
    }
}

impl QuestionSpec {
    /// Convert this spec into a runtime [`Question`].
    pub fn to_question(&self) -> Result<Question, String> {
        let question_type = match self.r#type {
            QuestionTypeSpec::Freeform => QuestionType::Freeform,
            QuestionTypeSpec::YesNo => QuestionType::YesNo,
            QuestionTypeSpec::Confirm => QuestionType::Confirm,
            QuestionTypeSpec::SingleSelect => QuestionType::SingleSelect,
            QuestionTypeSpec::MultiSelect => QuestionType::MultiSelect,
        };

        // Build runtime options with auto-assigned keys.
        let options: Vec<QuestionOption> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, o)| QuestionOption {
                key: option_key(i),
                label: o.label.clone(),
                description: o.description.clone(),
            })
            .collect();

        // Validate that choice-based questions have at least one option.
        if matches!(
            question_type,
            QuestionType::SingleSelect | QuestionType::MultiSelect
        ) && options.is_empty()
        {
            return Err(format!(
                "question '{}' is {question_type} but has no options",
                self.question
            ));
        }

        // Parse the default value, if specified.
        let default = self
            .default
            .as_deref()
            .map(|d| parse_default(d, &question_type, &options))
            .transpose()?
            .map(Answer::new);

        let mut q = match question_type {
            QuestionType::YesNo => Question::yes_no(&self.question),
            QuestionType::Confirm => Question::confirm(&self.question),
            QuestionType::Freeform => Question::freeform(&self.question),
            QuestionType::SingleSelect => Question::single_select(&self.question, options.clone()),
            QuestionType::MultiSelect => Question::multi_select(&self.question, options.clone()),
        };

        q.header = self.header.clone();
        q.default = default;
        q.store = self.store.clone();
        q.show_if = self.show_if.clone();
        q.finish_if = self.finish_if.clone();

        Ok(q)
    }
}

/// Parse a string default value into an [`AnswerValue`].
fn parse_default(
    default_str: &str,
    question_type: &QuestionType,
    options: &[QuestionOption],
) -> Result<AnswerValue, String> {
    let trimmed = default_str.trim();
    match question_type {
        QuestionType::YesNo | QuestionType::Confirm => match trimmed.to_lowercase().as_str() {
            "yes" | "y" | "true" => Ok(AnswerValue::Yes),
            "no" | "n" | "false" => Ok(AnswerValue::No),
            _ => Err(format!(
                "invalid default '{default_str}' for {question_type} question; \
                     expected 'yes' or 'no'"
            )),
        },
        QuestionType::Freeform => Ok(AnswerValue::Text(default_str.to_string())),
        QuestionType::SingleSelect => {
            let opt = options
                .iter()
                .find(|o| o.label.trim() == trimmed)
                .ok_or_else(|| {
                    format!("default '{default_str}' does not match any option label")
                })?;
            Ok(AnswerValue::Selected(opt.key.clone()))
        }
        QuestionType::MultiSelect => {
            if trimmed.is_empty() {
                return Ok(AnswerValue::MultiSelected(vec![]));
            }
            let labels: Vec<&str> = trimmed.split(',').map(str::trim).collect();
            let mut keys = Vec::with_capacity(labels.len());
            for label in &labels {
                let opt = options
                    .iter()
                    .find(|o| o.label.trim() == *label)
                    .ok_or_else(|| {
                        format!("default label '{label}' does not match any option label")
                    })?;
                keys.push(opt.key.clone());
            }
            Ok(AnswerValue::MultiSelected(keys))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // YAML deserialization roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn yaml_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
preamble: "Please answer these questions"
questions:
  - question: "What is your name?"
    header: "Name"
    store: user_name
  - question: "Continue?"
    type: yes-no
    default: "yes"
"#;
        let spec: InterviewSpec = serde_yaml::from_str(yaml)?;
        assert_eq!(
            spec.preamble.as_deref(),
            Some("Please answer these questions")
        );
        assert_eq!(spec.questions.len(), 2);
        assert_eq!(spec.questions[0].question, "What is your name?");
        assert_eq!(spec.questions[0].header.as_deref(), Some("Name"));
        assert_eq!(spec.questions[0].r#type, QuestionTypeSpec::Freeform);
        assert_eq!(spec.questions[0].store.as_deref(), Some("user_name"));
        assert_eq!(spec.questions[1].r#type, QuestionTypeSpec::YesNo);
        assert_eq!(spec.questions[1].default.as_deref(), Some("yes"));

        // Roundtrip: serialize back to YAML and re-parse.
        let yaml_out = serde_yaml::to_string(&spec)?;
        let spec2: InterviewSpec = serde_yaml::from_str(&yaml_out)?;
        assert_eq!(spec2.questions.len(), 2);
        assert_eq!(spec2.questions[0].question, "What is your name?");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // JSON deserialization roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "questions": [
                {
                    "question": "Pick a color",
                    "type": "single-select",
                    "options": [
                        {"label": "Red"},
                        {"label": "Blue", "description": "Like the sky"}
                    ],
                    "default": "Blue"
                }
            ]
        }"#;
        let spec: InterviewSpec = serde_json::from_str(json)?;
        assert!(spec.preamble.is_none());
        assert_eq!(spec.questions.len(), 1);
        assert_eq!(spec.questions[0].r#type, QuestionTypeSpec::SingleSelect);
        assert_eq!(spec.questions[0].options.len(), 2);
        assert_eq!(
            spec.questions[0].options[1].description.as_deref(),
            Some("Like the sky")
        );

        let json_out = serde_json::to_string(&spec)?;
        let spec2: InterviewSpec = serde_json::from_str(&json_out)?;
        assert_eq!(spec2.questions[0].options[0].label, "Red");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // to_interview() builds correct question types, options, headers, defaults
    // -----------------------------------------------------------------------

    #[test]
    fn to_interview_correct_question_types() -> Result<(), String> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Continue?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Pick one".into(),
                    header: Some("Choice".into()),
                    r#type: QuestionTypeSpec::SingleSelect,
                    options: vec![
                        OptionSpec {
                            label: "Alpha".into(),
                            description: None,
                        },
                        OptionSpec {
                            label: "Beta".into(),
                            description: Some("second".into()),
                        },
                    ],
                    default: Some("Beta".into()),
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };
        let interview = spec.to_interview("test-stage")?;
        assert_eq!(interview.stage, "test-stage");
        assert_eq!(interview.questions.len(), 2);

        // First question
        assert_eq!(interview.questions[0].r#type, QuestionType::YesNo);
        assert_eq!(interview.questions[0].text, "Continue?");
        assert!(interview.questions[0].default.is_none());

        // Second question
        assert_eq!(interview.questions[1].r#type, QuestionType::SingleSelect);
        assert_eq!(interview.questions[1].header.as_deref(), Some("Choice"));
        assert_eq!(interview.questions[1].options.len(), 2);
        assert_eq!(interview.questions[1].options[0].key, "A");
        assert_eq!(interview.questions[1].options[0].label, "Alpha");
        assert_eq!(interview.questions[1].options[1].key, "B");
        assert_eq!(
            interview.questions[1].options[1].description.as_deref(),
            Some("second")
        );

        // Default resolved to key "B"
        let default = interview.questions[1]
            .default
            .as_ref()
            .expect("should have default");
        assert_eq!(default.value, AnswerValue::Selected("B".into()));

        Ok(())
    }

    // -----------------------------------------------------------------------
    // to_question() error: missing options for multi_choice
    // -----------------------------------------------------------------------

    #[test]
    fn to_question_error_missing_options_multi_choice() {
        let qs = QuestionSpec {
            question: "Pick one".into(),
            header: None,
            r#type: QuestionTypeSpec::SingleSelect,
            options: vec![],
            default: None,
            store: None,
            finish_if: None,
            show_if: None,
        };
        let err = qs
            .to_question()
            .expect_err("single-select with no options should fail");
        assert!(err.contains("no options"), "error was: {err}");
    }

    #[test]
    fn to_question_error_missing_options_multi_select() {
        let qs = QuestionSpec {
            question: "Pick some".into(),
            header: None,
            r#type: QuestionTypeSpec::MultiSelect,
            options: vec![],
            default: None,
            store: None,
            finish_if: None,
            show_if: None,
        };
        let err = qs
            .to_question()
            .expect_err("multi_select with no options should fail");
        assert!(err.contains("no options"), "error was: {err}");
    }

    // -----------------------------------------------------------------------
    // Deserialization error: unknown question_type
    // -----------------------------------------------------------------------

    #[test]
    fn deser_error_unknown_question_type() {
        let json = r#"{"question":"Q?","type":"radio_button"}"#;
        let result = serde_json::from_str::<QuestionSpec>(json);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // JSON schema generation
    // -----------------------------------------------------------------------

    #[test]
    fn json_schema_has_questions_property() -> Result<(), Box<dyn std::error::Error>> {
        let schema = schemars::schema_for!(InterviewSpec);
        let json = serde_json::to_value(&schema)?;
        let properties = json
            .get("properties")
            .expect("schema should have properties");
        assert!(
            properties.get("questions").is_some(),
            "missing 'questions' property"
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Single question creates Interview::single, multiple creates batch
    // -----------------------------------------------------------------------

    #[test]
    fn single_question_creates_single_interview() -> Result<(), String> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Proceed?".into(),
                header: None,
                r#type: QuestionTypeSpec::Confirm,
                options: vec![],
                default: None,
                store: None,
                finish_if: None,
                show_if: None,
            }],
        };
        let interview = spec.to_interview("s")?;
        assert_eq!(interview.questions.len(), 1);
        assert_eq!(interview.questions[0].r#type, QuestionType::Confirm);
        Ok(())
    }

    #[test]
    fn multiple_questions_creates_batch_interview() -> Result<(), String> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Q1?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Q2?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };
        let interview = spec.to_interview("s")?;
        assert_eq!(interview.questions.len(), 2);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Preamble is set on the interview
    // -----------------------------------------------------------------------

    #[test]
    fn preamble_set_on_interview() -> Result<(), String> {
        let spec = InterviewSpec {
            preamble: Some("Hello, please review.".into()),
            questions: vec![QuestionSpec {
                question: "OK?".into(),
                header: None,
                r#type: QuestionTypeSpec::Confirm,
                options: vec![],
                default: None,
                store: None,
                finish_if: None,
                show_if: None,
            }],
        };
        let interview = spec.to_interview("s")?;
        assert_eq!(interview.preamble.as_deref(), Some("Hello, please review."));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Option key assignment: A..Z then 27, 28, ...
    // -----------------------------------------------------------------------

    #[test]
    fn option_keys_a_through_z_then_numeric() {
        // First 26 should be A..Z
        for i in 0..26 {
            let expected = String::from((b'A' + i as u8) as char);
            assert_eq!(option_key(i), expected);
        }
        // Beyond 26, use numeric (index + 1)
        assert_eq!(option_key(26), "27");
        assert_eq!(option_key(27), "28");
        assert_eq!(option_key(99), "100");
    }

    #[test]
    fn to_question_assigns_keys_for_many_options() -> Result<(), String> {
        let options: Vec<OptionSpec> = (0..28)
            .map(|i| OptionSpec {
                label: format!("Option {i}"),
                description: None,
            })
            .collect();
        let qs = QuestionSpec {
            question: "Pick".into(),
            header: None,
            r#type: QuestionTypeSpec::SingleSelect,
            options,
            default: None,
            store: None,
            finish_if: None,
            show_if: None,
        };
        let q = qs.to_question()?;
        assert_eq!(q.options[0].key, "A");
        assert_eq!(q.options[25].key, "Z");
        assert_eq!(q.options[26].key, "27");
        assert_eq!(q.options[27].key, "28");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Default parsing for all question types
    // -----------------------------------------------------------------------

    #[test]
    fn default_yes_no() -> Result<(), String> {
        for (input, expected) in [
            ("yes", AnswerValue::Yes),
            ("y", AnswerValue::Yes),
            ("true", AnswerValue::Yes),
            ("YES", AnswerValue::Yes),
            ("no", AnswerValue::No),
            ("n", AnswerValue::No),
            ("false", AnswerValue::No),
            ("NO", AnswerValue::No),
        ] {
            let result = parse_default(input, &QuestionType::YesNo, &[])?;
            assert_eq!(result, expected, "input: {input}");
        }
        Ok(())
    }

    #[test]
    fn default_yes_no_invalid() {
        let result = parse_default("maybe", &QuestionType::YesNo, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn default_confirm() -> Result<(), String> {
        let result = parse_default("yes", &QuestionType::Confirm, &[])?;
        assert_eq!(result, AnswerValue::Yes);
        let result = parse_default("no", &QuestionType::Confirm, &[])?;
        assert_eq!(result, AnswerValue::No);
        Ok(())
    }

    #[test]
    fn default_freeform() -> Result<(), String> {
        let result = parse_default("hello world", &QuestionType::Freeform, &[])?;
        assert_eq!(result, AnswerValue::Text("hello world".into()));
        Ok(())
    }

    #[test]
    fn default_freeform_preserves_whitespace() -> Result<(), String> {
        let result = parse_default("  padded  ", &QuestionType::Freeform, &[])?;
        assert_eq!(result, AnswerValue::Text("  padded  ".into()));
        Ok(())
    }

    #[test]
    fn default_multi_choice() -> Result<(), String> {
        let options = vec![
            QuestionOption {
                key: "A".into(),
                label: "Red".into(),
                description: None,
            },
            QuestionOption {
                key: "B".into(),
                label: "Blue".into(),
                description: None,
            },
        ];
        let result = parse_default("Blue", &QuestionType::SingleSelect, &options)?;
        assert_eq!(result, AnswerValue::Selected("B".into()));
        Ok(())
    }

    #[test]
    fn default_multi_choice_no_match() {
        let options = vec![QuestionOption {
            key: "A".into(),
            label: "Red".into(),
            description: None,
        }];
        let result = parse_default("Green", &QuestionType::SingleSelect, &options);
        assert!(result.is_err());
    }

    #[test]
    fn default_multi_select() -> Result<(), String> {
        let options = vec![
            QuestionOption {
                key: "A".into(),
                label: "Lint".into(),
                description: None,
            },
            QuestionOption {
                key: "B".into(),
                label: "Build".into(),
                description: None,
            },
            QuestionOption {
                key: "C".into(),
                label: "Test".into(),
                description: None,
            },
        ];
        let result = parse_default("Lint, Build", &QuestionType::MultiSelect, &options)?;
        assert_eq!(
            result,
            AnswerValue::MultiSelected(vec!["A".into(), "B".into()])
        );
        Ok(())
    }

    #[test]
    fn default_multi_select_empty() -> Result<(), String> {
        let result = parse_default("", &QuestionType::MultiSelect, &[])?;
        assert_eq!(result, AnswerValue::MultiSelected(vec![]));
        let result = parse_default("  ", &QuestionType::MultiSelect, &[])?;
        assert_eq!(result, AnswerValue::MultiSelected(vec![]));
        Ok(())
    }

    #[test]
    fn default_multi_select_no_match() {
        let options = vec![QuestionOption {
            key: "A".into(),
            label: "Lint".into(),
            description: None,
        }];
        let result = parse_default("Deploy", &QuestionType::MultiSelect, &options);
        assert!(result.is_err());
    }

    #[test]
    fn default_trims_whitespace() -> Result<(), String> {
        let result = parse_default("  yes  ", &QuestionType::YesNo, &[])?;
        assert_eq!(result, AnswerValue::Yes);

        let options = vec![QuestionOption {
            key: "A".into(),
            label: "Blue".into(),
            description: None,
        }];
        let result = parse_default("  Blue  ", &QuestionType::SingleSelect, &options)?;
        assert_eq!(result, AnswerValue::Selected("A".into()));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // store field preserved through serde roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn store_field_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let spec = QuestionSpec {
            question: "Name?".into(),
            header: None,
            r#type: QuestionTypeSpec::Freeform,
            options: vec![],
            default: None,
            store: Some("user_name".into()),
            finish_if: None,
            show_if: None,
        };
        let json = serde_json::to_string(&spec)?;
        assert!(json.contains("user_name"));

        let spec2: QuestionSpec = serde_json::from_str(&json)?;
        assert_eq!(spec2.store.as_deref(), Some("user_name"));
        Ok(())
    }

    #[test]
    fn store_field_none_omitted() -> Result<(), Box<dyn std::error::Error>> {
        let spec = QuestionSpec {
            question: "Name?".into(),
            header: None,
            r#type: QuestionTypeSpec::Freeform,
            options: vec![],
            default: None,
            store: None,
            finish_if: None,
            show_if: None,
        };
        let json = serde_json::to_string(&spec)?;
        assert!(!json.contains("store"));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // QuestionTypeSpec default is Freeform
    // -----------------------------------------------------------------------

    #[test]
    fn question_type_spec_default_is_freeform() {
        assert_eq!(QuestionTypeSpec::default(), QuestionTypeSpec::Freeform);
    }

    // -----------------------------------------------------------------------
    // Full end-to-end: YAML spec to Interview with defaults
    // -----------------------------------------------------------------------

    #[test]
    fn yaml_to_interview_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
preamble: "Review configuration"
questions:
  - question: "Enable debug mode?"
    type: yes-no
    default: "yes"
  - question: "Select format"
    header: "Format"
    type: single-select
    options:
      - label: "JSON"
      - label: "YAML"
        description: "Human-readable"
    default: "YAML"
  - question: "Select steps"
    type: multi-select
    options:
      - label: "Lint"
      - label: "Build"
      - label: "Test"
    default: "Lint, Test"
  - question: "Any comments?"
    default: "No comments"
    store: comments
"#;
        let spec: InterviewSpec = serde_yaml::from_str(yaml)?;
        let interview = spec.to_interview("config-stage")?;

        assert_eq!(interview.preamble.as_deref(), Some("Review configuration"));
        assert_eq!(interview.stage, "config-stage");
        assert_eq!(interview.questions.len(), 4);

        // Q0: yes_no with default Yes
        assert_eq!(interview.questions[0].r#type, QuestionType::YesNo);
        assert_eq!(
            interview.questions[0].default.as_ref().map(|a| &a.value),
            Some(&AnswerValue::Yes)
        );

        // Q1: single-select with default "B" (YAML)
        assert_eq!(interview.questions[1].r#type, QuestionType::SingleSelect);
        assert_eq!(interview.questions[1].header.as_deref(), Some("Format"));
        assert_eq!(
            interview.questions[1].default.as_ref().map(|a| &a.value),
            Some(&AnswerValue::Selected("B".into()))
        );

        // Q2: multi_select with default ["A", "C"] (Lint, Test)
        assert_eq!(interview.questions[2].r#type, QuestionType::MultiSelect);
        assert_eq!(
            interview.questions[2].default.as_ref().map(|a| &a.value),
            Some(&AnswerValue::MultiSelected(vec!["A".into(), "C".into()]))
        );

        // Q3: freeform with default text
        assert_eq!(interview.questions[3].r#type, QuestionType::Freeform);
        assert_eq!(
            interview.questions[3].default.as_ref().map(|a| &a.value),
            Some(&AnswerValue::Text("No comments".into()))
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // is_conditional()
    // -----------------------------------------------------------------------

    #[test]
    fn is_conditional_false_for_plain_spec() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::Freeform,
                options: vec![],
                default: None,
                store: None,
                finish_if: None,
                show_if: None,
            }],
        };
        assert!(!spec.is_conditional());
    }

    #[test]
    fn is_conditional_true_with_finish_if() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::YesNo,
                options: vec![],
                default: None,
                store: None,
                finish_if: Some("no".into()),
                show_if: None,
            }],
        };
        assert!(spec.is_conditional());
    }

    #[test]
    fn is_conditional_true_with_show_if() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Q1?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("q1".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Q2?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: Some("q1 == yes".into()),
                },
            ],
        };
        assert!(spec.is_conditional());
    }

    // -----------------------------------------------------------------------
    // validate()
    // -----------------------------------------------------------------------

    #[test]
    fn validate_valid_conditional_spec() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Target?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("target".into()),
                    finish_if: None,
                    show_if: Some("approved == yes".into()),
                },
            ],
        };
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn validate_show_if_references_future_store() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Q1?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: Some("future_key == yes".into()),
                },
                QuestionSpec {
                    question: "Q2?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("future_key".into()),
                    finish_if: None,
                    show_if: None,
                },
            ],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors.iter().any(|e| e.contains("future_key")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_show_if_bad_syntax() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::Freeform,
                options: vec![],
                default: None,
                store: None,
                finish_if: None,
                show_if: Some("no operator".into()),
            }],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors.iter().any(|e| e.contains("invalid show-if")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_finish_if_freeform_rejected() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::Freeform,
                options: vec![],
                default: None,
                store: None,
                finish_if: Some("anything".into()),
                show_if: None,
            }],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors
                .iter()
                .any(|e| e.contains("not supported for freeform")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_finish_if_multi_select_rejected() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::MultiSelect,
                options: vec![OptionSpec {
                    label: "A".into(),
                    description: None,
                }],
                default: None,
                store: None,
                finish_if: Some("A".into()),
                show_if: None,
            }],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors
                .iter()
                .any(|e| e.contains("not supported for multi_select")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_finish_if_yes_no_bad_value() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::YesNo,
                options: vec![],
                default: None,
                store: None,
                finish_if: Some("maybe".into()),
                show_if: None,
            }],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors.iter().any(|e| e.contains("must be 'yes' or 'no'")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_finish_if_mc_unknown_label() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "Q?".into(),
                header: None,
                r#type: QuestionTypeSpec::SingleSelect,
                options: vec![OptionSpec {
                    label: "Alpha".into(),
                    description: None,
                }],
                default: None,
                store: None,
                finish_if: Some("Beta".into()),
                show_if: None,
            }],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors
                .iter()
                .any(|e| e.contains("does not match any option label")),
            "errors: {errors:?}"
        );
    }

    #[test]
    fn validate_duplicate_store_keys() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Q1?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("dup".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Q2?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("dup".into()),
                    finish_if: None,
                    show_if: None,
                },
            ],
        };
        let errors = spec.validate().expect_err("should fail validation");
        assert!(
            errors.iter().any(|e| e.contains("duplicate store key")),
            "errors: {errors:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Conditional fields serde roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn conditional_fields_yaml_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
questions:
  - question: "Approve?"
    question_type: yes_no
    store: approved
    finish_if: "no"
  - question: "Why not?"
    question_type: freeform
    show_if: "approved == no"
"#;
        let spec: InterviewSpec = serde_yaml::from_str(yaml)?;
        assert_eq!(spec.questions[0].finish_if.as_deref(), Some("no"));
        assert_eq!(spec.questions[1].show_if.as_deref(), Some("approved == no"));
        assert!(spec.is_conditional());

        let yaml_out = serde_yaml::to_string(&spec)?;
        let spec2: InterviewSpec = serde_yaml::from_str(&yaml_out)?;
        assert_eq!(spec2.questions[0].finish_if.as_deref(), Some("no"));
        assert_eq!(
            spec2.questions[1].show_if.as_deref(),
            Some("approved == no")
        );
        Ok(())
    }

    #[test]
    fn conditional_fields_omitted_when_none() -> Result<(), Box<dyn std::error::Error>> {
        let spec = QuestionSpec {
            question: "Q?".into(),
            header: None,
            r#type: QuestionTypeSpec::Freeform,
            options: vec![],
            default: None,
            store: None,
            finish_if: None,
            show_if: None,
        };
        let json = serde_json::to_string(&spec)?;
        assert!(!json.contains("finish-if"));
        assert!(!json.contains("show-if"));
        Ok(())
    }
}
