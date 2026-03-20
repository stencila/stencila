//! CLI interviewer (§6.4).
//!
//! Presents interview questions on the terminal via `dialoguer` prompts.
//! Returns [`AnswerValue::Skipped`] when stdin is not a TTY.

use std::io::IsTerminal;

use async_trait::async_trait;

use stencila_interviews::interviewer::{
    Answer, AnswerValue, InterviewError, Interviewer, Question, QuestionType, parse_answer_text,
};

/// An interviewer that presents questions on the terminal using `dialoguer`.
///
/// Supports yes/no, confirm, multiple-choice (with accelerator keys),
/// and freeform text prompts. When `timeout_seconds` is set on a question,
/// the blocking stdin read is wrapped in a `tokio::time::timeout`.
///
/// If stdin is not a TTY (e.g. piped input in CI), all questions return
/// [`AnswerValue::Skipped`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CliInterviewer;

impl CliInterviewer {
    fn ask_blocking(question: &Question) -> Answer {
        if !std::io::stdin().is_terminal() {
            return Answer::new(AnswerValue::Skipped);
        }

        match question.r#type {
            QuestionType::YesNo | QuestionType::Confirm => ask_yes_no(question),
            QuestionType::SingleSelect => {
                if question.options.is_empty() {
                    ask_freeform(question)
                } else {
                    ask_select(question)
                }
            }
            QuestionType::MultiSelect => {
                if question.options.is_empty() {
                    ask_freeform(question)
                } else {
                    ask_multi_select(question)
                }
            }
            QuestionType::Freeform => ask_freeform(question),
        }
    }
}

#[async_trait]
impl Interviewer for CliInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        let q = question.clone();

        if let Some(duration) = question.timeout_duration() {
            let secs = duration.as_secs_f64();
            match tokio::time::timeout(
                duration,
                tokio::task::spawn_blocking(move || Self::ask_blocking(&q)),
            )
            .await
            {
                Ok(Ok(answer)) => Ok(answer),
                Ok(Err(join_error)) => Err(InterviewError::BackendFailure(format!(
                    "CLI prompt task failed: {join_error}"
                ))),
                Err(_timeout) => {
                    // The spawn_blocking task may still be waiting on stdin.
                    // There is no way to cancel a blocking read; the detached
                    // task will be cleaned up when the process exits.
                    tracing::warn!("CLI prompt timed out after {secs}s; blocking reader detached");
                    Ok(question.timeout_answer())
                }
            }
        } else {
            match tokio::task::spawn_blocking(move || Self::ask_blocking(&q)).await {
                Ok(answer) => Ok(answer),
                Err(join_error) => Err(InterviewError::BackendFailure(format!(
                    "CLI prompt task failed: {join_error}"
                ))),
            }
        }
    }

    #[allow(clippy::print_stderr)]
    fn inform(&self, message: &str, stage: &str) {
        if stage.is_empty() {
            eprintln!("ℹ️  {message}");
        } else {
            eprintln!("ℹ️  [{stage}] {message}");
        }
    }
}

fn ask_yes_no(question: &Question) -> Answer {
    let default_yes = question
        .default
        .as_ref()
        .is_some_and(|a| a.value == AnswerValue::Yes);

    let result = dialoguer::Confirm::new()
        .with_prompt(format!("❔ {}", &question.text))
        .default(default_yes)
        .interact();

    match result {
        Ok(true) => Answer::new(AnswerValue::Yes),
        Ok(false) => Answer::new(AnswerValue::No),
        Err(_) => Answer::new(AnswerValue::Skipped),
    }
}

fn ask_select(question: &Question) -> Answer {
    let items: Vec<&str> = question
        .options
        .iter()
        .map(|opt| opt.label.as_str())
        .collect();

    let result = dialoguer::Select::new()
        .with_prompt(format!("❔ {}", &question.text))
        .items(&items)
        .default(0)
        .interact();

    match result {
        Ok(index) => {
            if let Some(opt) = question.options.get(index) {
                Answer::with_option(AnswerValue::Selected(opt.key.clone()), opt.clone())
            } else {
                Answer::new(AnswerValue::Skipped)
            }
        }
        Err(_) => Answer::new(AnswerValue::Skipped),
    }
}

fn ask_multi_select(question: &Question) -> Answer {
    let items: Vec<&str> = question
        .options
        .iter()
        .map(|opt| opt.label.as_str())
        .collect();

    let result = dialoguer::MultiSelect::new()
        .with_prompt(format!("❔ {}", &question.text))
        .items(&items)
        .interact();

    match result {
        Ok(indices) => {
            let keys: Vec<String> = indices
                .iter()
                .filter_map(|&i| question.options.get(i).map(|o| o.key.clone()))
                .collect();
            if keys.is_empty() {
                Answer::new(AnswerValue::Skipped)
            } else {
                Answer::new(AnswerValue::MultiSelected(keys))
            }
        }
        Err(_) => Answer::new(AnswerValue::Skipped),
    }
}

fn ask_freeform(question: &Question) -> Answer {
    let mut input = dialoguer::Input::<String>::new().with_prompt(format!("❔ {}", &question.text));

    if let Some(ref default) = question.default
        && let AnswerValue::Text(ref text) = default.value
    {
        input = input.default(text.clone());
    }

    match input.interact_text() {
        Ok(text) => parse_answer_text(&text, question),
        Err(_) => Answer::new(AnswerValue::Skipped),
    }
}
