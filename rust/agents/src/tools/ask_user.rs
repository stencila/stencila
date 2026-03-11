use std::sync::Arc;

use serde_json::Value;
use stencila_interviews::conduct::conduct_conditional;
use stencila_interviews::interviewer::{AnswerValue, Interview, Interviewer};
use stencila_interviews::spec::InterviewSpec;
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput, ToolRegistry};

pub fn definition() -> ToolDefinition {
    let schema = schemars::schema_for!(InterviewSpec);

    ToolDefinition {
        name: "ask_user".into(),
        description: "Ask the user one or more questions and wait for their responses. \
            Use this when you need clarification, confirmation, or choices from the user \
            before proceeding. Supports freeform text, yes/no, single-select, and \
            multi-select questions. Multiple questions are presented together as a form \
            where the frontend supports it; otherwise they are presented sequentially."
            .into(),
        parameters: serde_json::to_value(schema)
            .expect("InterviewSpec JSON Schema should always serialize"),
        strict: false,
    }
}

pub fn executor(interviewer: Arc<dyn Interviewer>) -> ToolExecutorFn {
    Box::new(
        move |args: Value, _env: &dyn crate::execution::ExecutionEnvironment| {
            let interviewer = interviewer.clone();
            Box::pin(async move {
                let spec: InterviewSpec =
                    serde_json::from_value(args).map_err(|e| AgentError::ValidationError {
                        reason: e.to_string(),
                    })?;

                if spec.questions.is_empty() {
                    return Err(AgentError::ValidationError {
                        reason: "questions array must contain at least one question".into(),
                    });
                }

                // Validate conditional features (show-if, finish-if)
                if let Err(errors) = spec.validate() {
                    return Err(AgentError::ValidationError {
                        reason: errors.join("; "),
                    });
                }

                // Conditional specs are conducted progressively (one
                // question at a time) so that show-if / finish-if can
                // be evaluated between questions.
                if spec.is_conditional() {
                    let result = conduct_conditional(&spec, interviewer.as_ref(), "ask_user")
                        .await
                        .map_err(|e| AgentError::InterviewFailed {
                            message: e.to_string(),
                        })?;

                    let formatted = format_answers(&result.interview);
                    return Ok(ToolOutput::Text(formatted));
                }

                // Non-conditional: batch all questions together.
                let mut interview = spec
                    .to_interview("ask_user")
                    .map_err(|reason| AgentError::ValidationError { reason })?;

                let explicit_preamble = interview.preamble.clone();

                // When there is no explicit preamble and a single question
                // contains block-level markdown, auto-split the question text
                // into preamble + short trailing question so the rich content
                // renders correctly in the TUI.
                if explicit_preamble.is_none()
                    && interview.questions.len() == 1
                    && let Some((extracted_preamble, short_question)) =
                        extract_block_preamble(&interview.questions[0].text)
                {
                    interview.questions[0].text = short_question;
                    interview.preamble = Some(extracted_preamble);
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

/// Minimum question text length before we consider auto-splitting.
const BLOCK_SPLIT_MIN_LEN: usize = 120;

/// Maximum length for the extracted trailing question. If the tail is longer
/// than this it's probably still structured content, not a concise question.
const BLOCK_SPLIT_MAX_QUESTION_LEN: usize = 200;

/// Check whether `text` contains block-level markdown indicators
/// (numbered lists, bullet lists, headings, or multi-paragraph breaks).
fn has_block_markdown(text: &str) -> bool {
    // Multiple paragraphs
    if text.contains("\n\n") {
        return true;
    }
    for line in text.lines() {
        let trimmed = line.trim_start();
        // Headings (levels 1–6)
        if let Some(after_hashes) = trimmed.strip_prefix('#') {
            let after_hashes = after_hashes.trim_start_matches('#');
            // Must be 1–6 `#` chars total, followed by a space
            let hash_count = trimmed.len() - after_hashes.len();
            if hash_count <= 6 && after_hashes.starts_with(' ') {
                return true;
            }
        }
        // Bullet lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            return true;
        }
        // Numbered lists: digit(s) followed by `. ` or `) `
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with(". ") || rest.starts_with(") ") {
                return true;
            }
        }
    }
    false
}

/// When a question's text is long and contains block-level markdown, split it
/// into a preamble (the body) and a short trailing question (the last
/// sentence that ends with `?`). Returns `Some((preamble, question))` on
/// success, or `None` if the text doesn't qualify.
fn extract_block_preamble(text: &str) -> Option<(String, String)> {
    if text.len() < BLOCK_SPLIT_MIN_LEN || !has_block_markdown(text) {
        return None;
    }

    // Find the last `?` — the trailing question should be the sentence
    // ending at (or near) that position.
    let q_mark = text.rfind('?')?;

    // Walk backwards from the `?` to find the start of the trailing question
    // sentence. We look for the nearest sentence boundary: a newline, or a
    // period/exclamation followed by whitespace.
    let before = &text[..q_mark];
    let split_pos = before
        .rfind('\n')
        .map(|p| p + 1) // start of last line
        .or_else(|| {
            // Look for ". " or "! " sentence boundary
            before.rfind(". ").map(|p| p + 2)
        })
        .or_else(|| before.rfind("! ").map(|p| p + 2))
        .unwrap_or(0);

    let preamble = text[..split_pos].trim();
    let question = text[split_pos..].trim();

    // Reject if either part is empty, the preamble is too short to be
    // meaningful, the trailing question is too long (probably still structured
    // content), or the trailing question contains embedded newlines.
    if preamble.is_empty()
        || question.is_empty()
        || preamble.len() < 20
        || question.len() > BLOCK_SPLIT_MAX_QUESTION_LEN
        || question.contains('\n')
    {
        return None;
    }

    Some((preamble.to_string(), question.to_string()))
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
    use serde_json::json;
    use stencila_interviews::interviewer::{Answer, AnswerValue as AV, Question};
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
            "questions": [{"question": "Proceed?", "type": "yes-no"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[tokio::test]
    async fn single_confirm_auto_approve() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{"question": "Sure?", "type": "confirm"}]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[tokio::test]
    async fn single_single_select_auto_approve() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Pick a color:",
                "type": "single-select",
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
                {"question": "Deploy?", "type": "yes-no", "header": "Deployment"},
                {"question": "Which env?", "type": "single-select", "header": "Environment",
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
                "type": "single-select",
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
            "questions": [{"type": "freeform"}]
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
                {"question": "Q1?", "header": "Step 1", "type": "yes-no"},
                {"question": "Q2?", "header": "Step 2", "type": "yes-no"}
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
            "questions": [{"question": "Hello?", "type": "bogus"}]
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
                "type": "multi-select",
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

    #[tokio::test]
    async fn default_is_set_on_question() -> AgentResult<()> {
        let iv: Arc<dyn Interviewer> = Arc::new(AutoApproveInterviewer);
        let exec = executor(iv);
        let args = json!({
            "questions": [{
                "question": "Proceed?",
                "type": "yes-no",
                "default": "yes"
            }]
        });
        let result = exec(args, &env()).await?;
        assert_eq!(result.as_text(), "Yes");
        Ok(())
    }

    #[test]
    fn has_block_markdown_detects_numbered_lists() {
        assert!(has_block_markdown(
            "Here are items:\n1. First item\n2. Second item\nWhich do you want?"
        ));
        assert!(has_block_markdown("I found:\n10. A big item\nOK?"));
    }

    #[test]
    fn has_block_markdown_detects_bullet_lists() {
        assert!(has_block_markdown("Options:\n- alpha\n- beta\nPick one?"));
        assert!(has_block_markdown("Options:\n* alpha\n* beta\nPick one?"));
    }

    #[test]
    fn has_block_markdown_detects_headings() {
        assert!(has_block_markdown("# Summary\nSome text here, pick one?"));
        assert!(has_block_markdown("Intro\n## Details\nMore text, agree?"));
        assert!(has_block_markdown("#### Level 4 heading\nContent here?"));
        assert!(has_block_markdown("###### Level 6 heading\nContent?"));
        // Seven hashes is not a valid markdown heading
        assert!(!has_block_markdown("####### Not a heading"));
    }

    #[test]
    fn has_block_markdown_detects_paragraphs() {
        assert!(has_block_markdown(
            "First paragraph.\n\nSecond paragraph. OK?"
        ));
    }

    #[test]
    fn has_block_markdown_rejects_plain_text() {
        assert!(!has_block_markdown(
            "Should I convert all three, or would you prefer to pick a subset?"
        ));
        assert!(!has_block_markdown("Simple **bold** and *italic* text?"));
    }

    #[test]
    fn extract_block_preamble_splits_numbered_list() {
        let text = "I identified three workflows:\n\
                     1. `test-edge-weights` — Uses LLM prompts but edge-weight routing is purely an engine decision.\n\
                     2. `test-context-conditions` — Uses LLM prompts but context routing can be fully tested.\n\
                     3. `test-human-gates` — Already says no LLM calls.\n\
                     Should I convert all three, or would you prefer to pick a subset?";
        let (preamble, question) = extract_block_preamble(text).expect("should split");
        assert!(preamble.contains("1. `test-edge-weights`"));
        assert!(preamble.contains("3. `test-human-gates`"));
        assert_eq!(
            question,
            "Should I convert all three, or would you prefer to pick a subset?"
        );
    }

    #[test]
    fn extract_block_preamble_returns_none_for_short_text() {
        assert!(extract_block_preamble("Pick one?\n- a\n- b").is_none());
    }

    #[test]
    fn extract_block_preamble_returns_none_for_plain_text() {
        let text = "Should I convert all three workflows, or would you prefer to pick a subset of them to work on first? Let me know.";
        // Long enough but no block markdown
        assert!(extract_block_preamble(text).is_none());
    }

    #[test]
    fn extract_block_preamble_returns_none_without_question_mark() {
        let text = "I found these items:\n1. First\n2. Second\n3. Third\nPlease choose one from the list above.";
        // No trailing `?` — we need a question mark to identify the question sentence
        // This is >= 120 chars with block markdown but no `?`
        let padded = format!(
            "{text} Some extra padding to make it long enough for the threshold to apply here."
        );
        assert!(extract_block_preamble(&padded).is_none());
    }

    #[test]
    fn extract_block_preamble_rejects_long_trailing_question() {
        // When the trailing part after the last newline is very long (>200
        // chars), the split should be rejected because it's not a concise
        // question.
        let text = "Summary of findings:\n\
                     - Widget A has performance issues\n\
                     - Widget B has memory leaks\n\
                     Given these findings and considering the timeline, what should we prioritize? Also, should we consider refactoring Widget C which is deprecated and has several known vulnerabilities that need to be addressed before the next release cycle?";
        assert!(
            extract_block_preamble(text).is_none(),
            "should not split when trailing question exceeds max length"
        );
    }

    /// An `Interviewer` that captures the `Interview` it receives and
    /// auto-approves all questions, for asserting preamble promotion.
    struct CapturingInterviewer {
        captured: tokio::sync::Mutex<Option<Interview>>,
    }

    impl CapturingInterviewer {
        fn new() -> Self {
            Self {
                captured: tokio::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait::async_trait]
    impl Interviewer for CapturingInterviewer {
        async fn ask(
            &self,
            _question: &Question,
        ) -> Result<Answer, stencila_interviews::interviewer::InterviewError> {
            Ok(Answer::new(AV::Text("captured".to_string())))
        }

        async fn conduct(
            &self,
            interview: &mut Interview,
        ) -> Result<(), stencila_interviews::interviewer::InterviewError> {
            *self.captured.lock().await = Some(interview.clone());
            // Fill answers so format_answers works
            interview.answers = interview
                .questions
                .iter()
                .map(|_| Answer::new(AV::Text("captured".to_string())))
                .collect();
            Ok(())
        }
    }

    #[tokio::test]
    async fn auto_split_promotes_block_markdown_to_preamble() -> AgentResult<()> {
        let capturing = Arc::new(CapturingInterviewer::new());
        let iv: Arc<dyn Interviewer> = capturing.clone();
        let exec = executor(iv);
        let long_question = "I identified three workflows:\n\
                              1. `test-edge-weights` — Uses LLM prompts but edge-weight routing is purely an engine decision.\n\
                              2. `test-context-conditions` — Uses LLM prompts but context routing can be fully tested.\n\
                              3. `test-human-gates` — Already says no LLM calls.\n\
                              Should I convert all three, or would you prefer to pick a subset?";
        let args = json!({
            "questions": [{"question": long_question}]
        });
        exec(args, &env()).await?;

        let captured = capturing.captured.lock().await;
        let interview = captured
            .as_ref()
            .expect("interview should have been captured");

        assert!(
            interview.preamble.is_some(),
            "preamble should have been set by auto-split"
        );
        let preamble = interview.preamble.as_ref().expect("checked above");
        assert!(
            preamble.contains("1. `test-edge-weights`"),
            "preamble should contain the list items"
        );

        assert_eq!(interview.questions.len(), 1);
        assert_eq!(
            interview.questions[0].text,
            "Should I convert all three, or would you prefer to pick a subset?"
        );
        Ok(())
    }

    #[tokio::test]
    async fn explicit_preamble_prevents_auto_split() -> AgentResult<()> {
        let capturing = Arc::new(CapturingInterviewer::new());
        let iv: Arc<dyn Interviewer> = capturing.clone();
        let exec = executor(iv);
        let long_question = "I identified three workflows:\n\
                              1. `test-edge-weights` — Uses LLM prompts.\n\
                              2. `test-context-conditions` — Uses LLM prompts.\n\
                              3. `test-human-gates` — Already says no LLM calls.\n\
                              Should I convert all three?";
        let args = json!({
            "preamble": "Here is my analysis:",
            "questions": [{"question": long_question}]
        });
        exec(args, &env()).await?;

        let captured = capturing.captured.lock().await;
        let interview = captured
            .as_ref()
            .expect("interview should have been captured");

        // When explicit preamble is provided, it should be used as-is
        assert_eq!(interview.preamble.as_deref(), Some("Here is my analysis:"));
        // Question text should NOT be modified
        assert!(
            interview.questions[0]
                .text
                .contains("1. `test-edge-weights`")
        );
        Ok(())
    }
}
