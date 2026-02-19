//! Event emitters for CLI workflow run presentation.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use stencila_attractor::events::{EventEmitter, PipelineEvent};

static NO_COLOR: LazyLock<bool> =
    LazyLock::new(|| std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()));

fn color(code: &str, text: &str) -> String {
    if *NO_COLOR {
        text.to_string()
    } else {
        format!("\x1b[{code}m{text}\x1b[0m")
    }
}

fn truncate_to(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

fn single_line_preview(s: &str, max_chars: usize) -> String {
    let oneline: String = s
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .take(max_chars)
        .collect();
    if s.chars().count() > max_chars {
        format!("{}…", oneline.trim_end())
    } else {
        oneline
    }
}

// ---------------------------------------------------------------------------
// ProgressEventEmitter
// ---------------------------------------------------------------------------

struct StageState {
    bar: ProgressBar,
    started_at: Instant,
    agent_name: String,
}

pub struct ProgressEventEmitter {
    multi: MultiProgress,
    state: Mutex<HashMap<String, StageState>>,
}

impl ProgressEventEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl EventEmitter for ProgressEventEmitter {
    fn emit(&self, event: PipelineEvent) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match event {
            PipelineEvent::PipelineStarted { .. }
            | PipelineEvent::PipelineCompleted { .. }
            | PipelineEvent::PipelineFailed { .. }
            | PipelineEvent::CheckpointSaved { .. }
            | PipelineEvent::StageSessionEvent { .. } => {}

            PipelineEvent::StageStarted {
                ref node_id,
                stage_index,
            } => {
                if stage_index == 0 {
                    return;
                }
                let bar = self.multi.add(ProgressBar::new_spinner());
                bar.set_style(
                    ProgressStyle::with_template("{spinner:.cyan} {msg}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
                );
                bar.enable_steady_tick(std::time::Duration::from_millis(80));
                bar.set_message(format!("   {node_id}"));
                state.insert(
                    node_id.clone(),
                    StageState {
                        bar,
                        started_at: Instant::now(),
                        agent_name: String::new(),
                    },
                );
            }

            PipelineEvent::StagePrompt {
                ref node_id,
                ref agent_name,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.agent_name = agent_name.clone();
                    s.bar.set_message(format!("   {node_id} ({agent_name})"));
                }
            }

            PipelineEvent::StageResponse { .. } => {}

            PipelineEvent::StageCompleted { ref node_id, .. } => {
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let time_str = color("2", &format!("{elapsed:.1}s"));

                    let agent_part = if s.agent_name.is_empty() {
                        node_id.clone()
                    } else {
                        format!("{node_id} ({})", s.agent_name)
                    };

                    let line = format!("✅ {agent_part}  {time_str}");

                    s.bar.finish_with_message(line);
                }
            }

            PipelineEvent::StageFailed {
                ref node_id,
                ref reason,
                ..
            } => {
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let time_str = color("2", &format!("{elapsed:.1}s"));
                    let x = color("31", "✗");
                    let agent_part = if s.agent_name.is_empty() {
                        node_id.clone()
                    } else {
                        format!("{node_id} ({})", s.agent_name)
                    };
                    s.bar
                        .finish_with_message(format!("{x} {agent_part}  {time_str}"));
                    let err_msg = color("31", &format!("  Error: {reason}"));
                    let _ = self.multi.println(err_msg);
                }
            }

            PipelineEvent::StageRetrying {
                ref node_id,
                attempt,
                max_attempts,
                ..
            } => {
                if let Some(s) = state.get(node_id) {
                    let agent_part = if s.agent_name.is_empty() {
                        node_id.clone()
                    } else {
                        format!("{node_id} ({})", s.agent_name)
                    };
                    s.bar.set_message(format!(
                        "   {agent_part} retrying ({attempt}/{max_attempts})…"
                    ));
                }
            }

            PipelineEvent::ParallelStarted { ref node_id } => {
                let _ = self
                    .multi
                    .println(color("2", &format!("║ parallel started: {node_id}")));
            }
            PipelineEvent::ParallelCompleted { ref node_id } => {
                let _ = self
                    .multi
                    .println(color("2", &format!("║ parallel completed: {node_id}")));
            }
            PipelineEvent::ParallelBranchStarted {
                ref node_id,
                branch_index,
            } => {
                let _ = self.multi.println(color(
                    "2",
                    &format!("║ branch {branch_index} started: {node_id}"),
                ));
            }
            PipelineEvent::ParallelBranchCompleted {
                ref node_id,
                branch_index,
            } => {
                let _ = self.multi.println(color(
                    "2",
                    &format!("║ branch {branch_index} completed: {node_id}"),
                ));
            }
            PipelineEvent::ParallelBranchFailed {
                ref node_id,
                branch_index,
                ref reason,
            } => {
                let _ = self.multi.println(color(
                    "2",
                    &format!("║ branch {branch_index} failed: {node_id}: {reason}"),
                ));
            }

            PipelineEvent::InterviewQuestionAsked { ref node_id } => {
                let _ = self.multi.println(format!("? question asked at {node_id}"));
            }
            PipelineEvent::InterviewAnswerReceived { ref node_id } => {
                let _ = self
                    .multi
                    .println(format!("✓ answer received at {node_id}"));
            }
            PipelineEvent::InterviewTimedOut { ref node_id } => {
                let _ = self
                    .multi
                    .println(format!("⏱ interview timed out at {node_id}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// VerboseEventEmitter
// ---------------------------------------------------------------------------

struct VerboseState {
    stages: HashMap<String, Instant>,
    pipeline_started: Option<Instant>,
}

pub struct VerboseEventEmitter {
    state: Mutex<VerboseState>,
}

impl VerboseEventEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VerboseState {
                stages: HashMap::new(),
                pipeline_started: None,
            }),
        }
    }
}

#[allow(clippy::print_stderr)]
impl EventEmitter for VerboseEventEmitter {
    fn emit(&self, event: PipelineEvent) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match event {
            PipelineEvent::PipelineStarted { .. } => {
                state.pipeline_started = Some(Instant::now());
            }

            PipelineEvent::PipelineCompleted { .. } | PipelineEvent::PipelineFailed { .. } => {}

            PipelineEvent::StageStarted {
                stage_index,
                ref node_id,
            } => {
                if stage_index == 0 {
                    return;
                }
                state.stages.insert(node_id.clone(), Instant::now());
                let bold_name = color("1", node_id);
                eprintln!("├─ {bold_name}");
            }

            PipelineEvent::StagePrompt {
                ref node_id,
                ref prompt,
                ref agent_name,
                ..
            } => {
                let preview = truncate_to(prompt, 100);
                let label_agent = color("2", "Agent:");
                eprintln!("│  {label_agent} {agent_name}");
                let _ = node_id;
                let label_prompt = color("2", "Prompt:");
                eprintln!("│  {label_prompt} {preview}");
            }

            PipelineEvent::StageSessionEvent { .. } => {}

            PipelineEvent::StageResponse { ref response, .. } => {
                let preview = truncate_to(response, 100);
                let label = color("2", "Response:");
                eprintln!("│  {label} {preview}");
            }

            PipelineEvent::StageCompleted {
                stage_index,
                ref node_id,
                outcome,
                ..
            } => {
                if stage_index == 0 {
                    return;
                }

                let elapsed = state
                    .stages
                    .remove(node_id)
                    .map(|t| t.elapsed().as_secs_f64())
                    .unwrap_or(0.0);
                let label = color("2", "Outcome:");
                let tick = color("32", "✔");
                let time_str = color("2", &format!("({elapsed:.1}s)"));
                eprintln!("│  {label} {tick} {} {time_str}", outcome.status.as_str());
                eprintln!("│");
            }

            PipelineEvent::StageFailed {
                ref node_id,
                ref reason,
                ..
            } => {
                state.stages.remove(node_id);
                let label = color("2", "Outcome:");
                let x = color("31", "✗");
                eprintln!("│  {label} {x} failed: {reason}");
                eprintln!("│");
            }

            PipelineEvent::StageRetrying {
                attempt,
                max_attempts,
                ..
            } => {
                eprintln!("│  ⟳ retrying ({attempt}/{max_attempts})");
            }

            PipelineEvent::CheckpointSaved { .. } => {}

            PipelineEvent::ParallelStarted { ref node_id } => {
                eprintln!("│  ║ parallel started: {node_id}");
            }
            PipelineEvent::ParallelCompleted { ref node_id } => {
                eprintln!("│  ║ parallel completed: {node_id}");
            }
            PipelineEvent::ParallelBranchStarted {
                ref node_id,
                branch_index,
            } => {
                eprintln!("│  ║ branch {branch_index} started: {node_id}");
            }
            PipelineEvent::ParallelBranchCompleted {
                ref node_id,
                branch_index,
            } => {
                eprintln!("│  ║ branch {branch_index} completed: {node_id}");
            }
            PipelineEvent::ParallelBranchFailed {
                ref node_id,
                branch_index,
                ref reason,
            } => {
                eprintln!("│  ║ branch {branch_index} failed: {node_id}: {reason}");
            }

            PipelineEvent::InterviewQuestionAsked { ref node_id } => {
                eprintln!("│  question asked at {node_id}");
            }
            PipelineEvent::InterviewAnswerReceived { ref node_id } => {
                eprintln!("│  answer received at {node_id}");
            }
            PipelineEvent::InterviewTimedOut { ref node_id } => {
                eprintln!("│  interview timed out at {node_id}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// PlainEventEmitter
// ---------------------------------------------------------------------------

struct PlainStageState {
    started_at: Instant,
    agent_name: String,
    response_preview: String,
}

pub struct PlainEventEmitter {
    state: Mutex<HashMap<String, PlainStageState>>,
}

impl PlainEventEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

#[allow(clippy::print_stderr)]
impl EventEmitter for PlainEventEmitter {
    fn emit(&self, event: PipelineEvent) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match event {
            PipelineEvent::StageStarted { ref node_id, .. } => {
                state.insert(
                    node_id.clone(),
                    PlainStageState {
                        started_at: Instant::now(),
                        agent_name: String::new(),
                        response_preview: String::new(),
                    },
                );
            }

            PipelineEvent::StagePrompt {
                ref node_id,
                ref agent_name,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.agent_name = agent_name.clone();
                }
            }

            PipelineEvent::StageResponse {
                ref node_id,
                ref response,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.response_preview = single_line_preview(response, 60);
                }
            }

            PipelineEvent::StageCompleted { ref node_id, .. } => {
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let agent_part = if s.agent_name.is_empty() {
                        node_id.clone()
                    } else {
                        format!("{node_id} ({})", s.agent_name)
                    };
                    let response_part = if s.response_preview.is_empty() {
                        String::new()
                    } else {
                        format!(" → {}", s.response_preview)
                    };
                    eprintln!("  ✓ {agent_part}{response_part}  {elapsed:.1}s");
                }
            }

            PipelineEvent::StageFailed {
                ref node_id,
                ref reason,
                ..
            } => {
                if let Some(s) = state.remove(node_id) {
                    let agent_part = if s.agent_name.is_empty() {
                        node_id.clone()
                    } else {
                        format!("{node_id} ({})", s.agent_name)
                    };
                    eprintln!("  ✗ {agent_part}: {reason}");
                }
            }

            PipelineEvent::PipelineStarted { .. }
            | PipelineEvent::PipelineCompleted { .. }
            | PipelineEvent::PipelineFailed { .. }
            | PipelineEvent::StageSessionEvent { .. }
            | PipelineEvent::StageRetrying { .. }
            | PipelineEvent::CheckpointSaved { .. }
            | PipelineEvent::ParallelStarted { .. }
            | PipelineEvent::ParallelCompleted { .. }
            | PipelineEvent::ParallelBranchStarted { .. }
            | PipelineEvent::ParallelBranchCompleted { .. }
            | PipelineEvent::ParallelBranchFailed { .. }
            | PipelineEvent::InterviewQuestionAsked { .. }
            | PipelineEvent::InterviewAnswerReceived { .. }
            | PipelineEvent::InterviewTimedOut { .. } => {}
        }
    }
}
