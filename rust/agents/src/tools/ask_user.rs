use std::sync::Arc;

use serde_json::{Value, json};
use stencila_interviews::interviewer::{
    Answer, AnswerValue, Interview, Interviewer, Question, QuestionOption, QuestionType,
};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput, ToolRegistry};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "ask_user".into(),
        description: "Ask the user one or more questions and wait for their responses. \
            Use this when you need clarification, confirmation, or choices from the user \
            before proceeding. Supports freeform text, yes/no, single-select, and \
            multi-select questions. Multiple questions are presented together as a form \
            where the frontend supports it; otherwise they are presented sequentially."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "preamble": {
                    "type": "string",
                    "description": "Optional Markdown content rendered before the questions as persistent interview context."
                },
                "questions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The question text to present."
                            },
                            "header": {
                                "type": "string",
                                "description": "Short label displayed above the question."
                            },
                            "question_type": {
                                "type": "string",
                                "enum": ["freeform", "yes_no", "multiple_choice", "multi_select", "confirmation"],
                                "description": "The type of question. Defaults to 'freeform'."
                            },
                            "options": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "description": "Display text for the option."
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Explanatory text shown alongside the label."
                                        }
                                    },
                                    "required": ["label"]
                                },
                                "description": "Choices for multiple_choice or multi_select questions."
                            },
                            "default": {
                                "type": "string",
                                "description": "Default answer used when the user skips or times out. \
                                    For yes_no and confirmation: 'yes' or 'no'. \
                                    For freeform: the default text. \
                                    For multiple_choice: the label of the default option. \
                                    For multi_select: comma-separated labels of default options \
                                    (empty string for no defaults). Option labels must not contain commas."
                            }
                        },
                        "required": ["question"]
                    },
                    "minItems": 1,
                    "description": "One or more questions to present to the user."
                }
            },
            "required": ["questions"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor(interviewer: Arc<dyn Interviewer>) -> ToolExecutorFn {
    Box::new(
        move |args: Value, _env: &dyn crate::execution::ExecutionEnvironment| {
            let interviewer = interviewer.clone();
            Box::pin(async move {
                let questions_val =
                    args.get("questions")
                        .and_then(Value::as_array)
                        .ok_or_else(|| AgentError::ValidationError {
                            reason: "missing required array parameter: questions".into(),
                        })?;

                if questions_val.is_empty() {
                    return Err(AgentError::ValidationError {
                        reason: "questions array must contain at least one question".into(),
                    });
                }

                let mut questions = Vec::with_capacity(questions_val.len());
                for item in questions_val {
                    let text = item
                        .get("question")
                        .and_then(Value::as_str)
                        .ok_or_else(|| AgentError::ValidationError {
                            reason: "each question must have a 'question' string field".into(),
                        })?;

                    let question_type = item
                        .get("question_type")
                        .and_then(Value::as_str)
                        .unwrap_or("freeform");

                    let mut q = match question_type {
                        "freeform" => Question::freeform(text),
                        "yes_no" => Question::yes_no(text),
                        "confirmation" => Question::confirmation(text),
                        "multiple_choice" => {
                            let options = parse_options(item)?;
                            Question::multiple_choice(text, options)
                        }
                        "multi_select" => {
                            let options = parse_options(item)?;
                            Question::multi_select(text, options)
                        }
                        other => {
                            return Err(AgentError::ValidationError {
                                reason: format!(
                                    "unknown question_type '{other}'; expected one of: \
                                     freeform, yes_no, multiple_choice, multi_select, confirmation"
                                ),
                            });
                        }
                    };

                    if let Some(header) = item.get("header").and_then(Value::as_str) {
                        q.header = Some(header.to_string());
                    }

                    if let Some(default_str) = item.get("default").and_then(Value::as_str) {
                        q.default = Some(Answer::new(parse_default(
                            default_str,
                            &q.question_type,
                            &q.options,
                        )?));
                    }

                    questions.push(q);
                }

                let preamble = args
                    .get("preamble")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_string);

                let mut interview = if questions.len() == 1 {
                    Interview::single(
                        questions
                            .into_iter()
                            .next()
                            .expect("length checked to be 1"),
                        "ask_user",
                    )
                } else {
                    Interview::batch(questions, "ask_user")
                };

                if let Some(preamble) = preamble {
                    interview = interview.with_preamble(preamble);
                }

                interviewer.conduct(&mut interview).await.map_err(|e| {
                    AgentError::InterviewFailed {
                        message: e.to_string(),
                    }
                })?;

                let formatted = format_answers(&interview);
                Ok(ToolOutput::Text(formatted))
            })
        },
    )
}

fn parse_options(item: &Value) -> AgentResult<Vec<QuestionOption>> {
    let opts_val = item
        .get("options")
        .and_then(Value::as_array)
        .ok_or_else(|| AgentError::ValidationError {
            reason: "multiple_choice and multi_select questions require an 'options' array".into(),
        })?;

    let mut options = Vec::with_capacity(opts_val.len());
    for (i, opt) in opts_val.iter().enumerate() {
        let label = opt.get("label").and_then(Value::as_str).ok_or_else(|| {
            AgentError::ValidationError {
                reason: "each option must have a 'label' string field".into(),
            }
        })?;
        let description = opt
            .get("description")
            .and_then(Value::as_str)
            .map(String::from);

        let key = if i < 26 {
            String::from(char::from(b'A' + i as u8))
        } else {
            format!("{}", i + 1)
        };

        options.push(QuestionOption {
            key,
            label: label.to_string(),
            description,
        });
    }
    Ok(options)
}

fn parse_default(
    default_str: &str,
    question_type: &QuestionType,
    options: &[QuestionOption],
) -> AgentResult<AnswerValue> {
    let trimmed = default_str.trim();
    match question_type {
        QuestionType::YesNo | QuestionType::Confirmation => match trimmed.to_lowercase().as_str() {
            "yes" | "y" | "true" => Ok(AnswerValue::Yes),
            "no" | "n" | "false" => Ok(AnswerValue::No),
            _ => Err(AgentError::ValidationError {
                reason: format!(
                    "invalid default '{default_str}' for {question_type} question; \
                     expected 'yes' or 'no'"
                ),
            }),
        },
        QuestionType::Freeform => Ok(AnswerValue::Text(default_str.to_string())),
        QuestionType::MultipleChoice => {
            let opt = options
                .iter()
                .find(|o| o.label.trim() == trimmed)
                .ok_or_else(|| AgentError::ValidationError {
                    reason: format!("default '{default_str}' does not match any option label"),
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
                    .ok_or_else(|| AgentError::ValidationError {
                        reason: format!("default label '{label}' does not match any option label"),
                    })?;
                keys.push(opt.key.clone());
            }
            Ok(AnswerValue::MultiSelected(keys))
        }
    }
}

fn format_answers(interview: &Interview) -> String {
    let single = interview.questions.len() == 1;
    let mut output = String::new();

    for (q, a) in interview.questions.iter().zip(interview.answers.iter()) {
        if !single {
            let heading = q.header.as_deref().unwrap_or(&q.text);
            output.push_str("## ");
            output.push_str(heading);
            output.push('\n');
        }

        let value_str = match &a.value {
            AnswerValue::Yes => "Yes".to_string(),
            AnswerValue::No => "No".to_string(),
            AnswerValue::Skipped => "[Skipped]".to_string(),
            AnswerValue::Timeout => "[Timed out]".to_string(),
            AnswerValue::Selected(key) => q
                .options
                .iter()
                .find(|o| o.key == *key)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| key.clone()),
            AnswerValue::MultiSelected(keys) => keys
                .iter()
                .map(|key| {
                    q.options
                        .iter()
                        .find(|o| o.key == *key)
                        .map(|o| o.label.clone())
                        .unwrap_or_else(|| key.clone())
                })
                .collect::<Vec<_>>()
                .join(", "),
            AnswerValue::Text(text) => text.clone(),
        };

        output.push_str(&value_str);
        output.push('\n');

        if !single {
            output.push('\n');
        }
    }

    output.truncate(output.trim_end().len());
    output
}

pub fn registered_tool(interviewer: Arc<dyn Interviewer>) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(interviewer))
}

pub fn register_ask_user_tool(
    registry: &mut ToolRegistry,
    interviewer: Arc<dyn Interviewer>,
) -> AgentResult<()> {
    registry.register(registered_tool(interviewer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_interviews::interviewer::{Answer, AnswerValue as AV};
    use stencila_interviews::interviewers::{AutoApproveInterviewer, QueueInterviewer};

    use crate::execution::LocalExecutionEnvironment;

    fn env() -> LocalExecutionEnvironment {
        LocalExecutionEnvironment::new(".")
    }

    fn text_answer(s: &str) -> Answer {
        Answer::new(AV::Text(s.to_string()))
    }

    #[tokio::test]
    async fn single_freeform_question() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(QueueInterviewer::new(vec![text_answer("Alice")]));
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "What is your name?"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Alice");
        Ok(())
    }

    #[tokio::test]
    async fn single_yes_no_auto_approve() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "Proceed?", "question_type": "yes_no"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[tokio::test]
    async fn single_confirmation_auto_approve() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "Sure?", "question_type": "confirmation"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[tokio::test]
    async fn single_multiple_choice_auto_approve() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Pick a color:",
                "question_type": "multiple_choice",
                "options": [
                    {"label": "Red"},
                    {"label": "Blue"},
                    {"label": "Green"}
                ]
            }]
        });
        let result = exec(args, &env()).await?;
        // AutoApproveInterviewer picks the first option
        assert_eq!(result.as_text(), "Red");
        Ok(())
    }

    #[tokio::test]
    async fn multiple_questions_batch() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [
                {"question": "Deploy?", "question_type": "yes_no", "header": "Deployment"},
                {"question": "Which env?", "question_type": "multiple_choice", "header": "Environment",
                 "options": [{"label": "Staging"}, {"label": "Production"}]}
            ]
        });
        let result = exec(args, &env()).await?;
        let text = result.as_text();
        assert!(text.contains("## Deployment"), "text: {text}");
        assert!(text.contains("Yes"), "text: {text}");
        assert!(text.contains("## Environment"), "text: {text}");
        assert!(text.contains("Staging"), "text: {text}");
        Ok(())
    }

    #[tokio::test]
    async fn options_get_sequential_keys() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Pick:",
                "question_type": "multiple_choice",
                "options": [
                    {"label": "First", "description": "The first one"},
                    {"label": "Second"},
                    {"label": "Third"}
                ]
            }]
        });
        let result = exec(args, &env()).await?;
        // AutoApproveInterviewer picks the first option (key "A", label "First")
        assert_eq!(result.as_text(), "First");
        Ok(())
    }

    #[tokio::test]
    async fn missing_questions_array_returns_validation_error() {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({});
        let result = exec(args, &env()).await;
        let err = result.expect_err("should be a validation error");
        assert!(matches!(err, AgentError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn missing_question_text_returns_validation_error() {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question_type": "freeform"}]
        });
        let result = exec(args, &env()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn default_question_type_is_freeform() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(QueueInterviewer::new(vec![text_answer("hello")]));
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "Say something:"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "hello");
        Ok(())
    }

    #[tokio::test]
    async fn header_is_set_on_question() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [
                {"question": "Q1?", "header": "Step 1", "question_type": "yes_no"},
                {"question": "Q2?", "header": "Step 2", "question_type": "yes_no"}
            ]
        });
        let result = exec(args, &env()).await?;
        let text = result.as_text();
        assert!(text.contains("## Step 1"));
        assert!(text.contains("## Step 2"));
        Ok(())
    }

    #[test]
    fn definition_validates() -> AgentResult<()> {
        definition().validate()?;
        Ok(())
    }

    #[test]
    fn register_ask_user_tool_works() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let mut registry = ToolRegistry::new();
        register_ask_user_tool(&mut registry, iv)?;
        assert!(registry.get("ask_user").is_some());
        Ok(())
    }

    #[tokio::test]
    async fn empty_questions_array_returns_validation_error() {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({ "questions": [] });
        let result = exec(args, &env()).await;
        let err = result.expect_err("should be a validation error");
        assert!(matches!(err, AgentError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn unknown_question_type_returns_validation_error() {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "Hello?", "question_type": "bogus"}]
        });
        let result = exec(args, &env()).await;
        let err = result.expect_err("should be a validation error");
        assert!(matches!(err, AgentError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn multi_select_formats_selected_labels() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(QueueInterviewer::new(vec![Answer::new(
            AV::MultiSelected(vec!["A".into(), "C".into()]),
        )]));
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Pick toppings:",
                "question_type": "multi_select",
                "options": [
                    {"label": "Cheese"},
                    {"label": "Pepperoni"},
                    {"label": "Mushrooms"}
                ]
            }]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Cheese, Mushrooms");
        Ok(())
    }

    #[test]
    fn parse_default_yes_no() -> AgentResult<()> {
        let av = parse_default("yes", &QuestionType::YesNo, &[])?;
        assert_eq!(av, AV::Yes);
        let av = parse_default("no", &QuestionType::YesNo, &[])?;
        assert_eq!(av, AV::No);
        let av = parse_default("Y", &QuestionType::Confirmation, &[])?;
        assert_eq!(av, AV::Yes);
        assert!(parse_default("maybe", &QuestionType::YesNo, &[]).is_err());
        Ok(())
    }

    #[test]
    fn parse_default_freeform() -> AgentResult<()> {
        let av = parse_default("hello world", &QuestionType::Freeform, &[])?;
        assert_eq!(av, AV::Text("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn parse_default_multiple_choice() -> AgentResult<()> {
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
        let av = parse_default("Blue", &QuestionType::MultipleChoice, &options)?;
        assert_eq!(av, AV::Selected("B".into()));
        assert!(parse_default("Green", &QuestionType::MultipleChoice, &options).is_err());
        Ok(())
    }

    #[test]
    fn parse_default_multi_select() -> AgentResult<()> {
        let options = vec![
            QuestionOption {
                key: "A".into(),
                label: "Lint".into(),
                description: None,
            },
            QuestionOption {
                key: "B".into(),
                label: "Test".into(),
                description: None,
            },
            QuestionOption {
                key: "C".into(),
                label: "Build".into(),
                description: None,
            },
        ];
        let av = parse_default("Lint, Build", &QuestionType::MultiSelect, &options)?;
        assert_eq!(av, AV::MultiSelected(vec!["A".into(), "C".into()]));
        Ok(())
    }

    #[test]
    fn parse_default_multi_select_empty() -> AgentResult<()> {
        let av = parse_default("", &QuestionType::MultiSelect, &[])?;
        assert_eq!(av, AV::MultiSelected(vec![]));
        let av = parse_default("  ", &QuestionType::MultiSelect, &[])?;
        assert_eq!(av, AV::MultiSelected(vec![]));
        Ok(())
    }

    #[test]
    fn parse_default_trims_whitespace() -> AgentResult<()> {
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
        let av = parse_default("  Blue  ", &QuestionType::MultipleChoice, &options)?;
        assert_eq!(av, AV::Selected("B".into()));
        let av = parse_default(" yes ", &QuestionType::YesNo, &[])?;
        assert_eq!(av, AV::Yes);
        Ok(())
    }

    #[tokio::test]
    async fn default_is_set_on_question() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Proceed?",
                "question_type": "yes_no",
                "default": "yes"
            }]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[test]
    fn option_keys_beyond_26_use_numeric() -> AgentResult<()> {
        let mut opts_json = Vec::new();
        for i in 0..30 {
            opts_json.push(json!({"label": format!("Option {i}")}));
        }
        let item = json!({
            "question": "Pick:",
            "question_type": "multiple_choice",
            "options": opts_json
        });
        let options = parse_options(&item)?;
        assert_eq!(options[0].key, "A");
        assert_eq!(options[25].key, "Z");
        assert_eq!(options[26].key, "27");
        assert_eq!(options[29].key, "30");
        Ok(())
    }
}
