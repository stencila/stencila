//! Event emitters for CLI workflow run presentation.

use std::collections::HashMap;
use std::io::Write;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

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
// Spinner — minimal inline spinner that overwrites a single stderr line
// ---------------------------------------------------------------------------

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

struct Spinner {
    message: std::sync::Arc<Mutex<String>>,
    handle: Option<std::thread::JoinHandle<()>>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Spinner {
    fn new(message: &str) -> Self {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let msg = std::sync::Arc::new(Mutex::new(message.to_string()));
        let stop2 = stop.clone();
        let msg2 = msg.clone();
        let handle = std::thread::spawn(move || {
            let mut idx = 0usize;
            while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
                let frame = color("36", SPINNER_FRAMES[idx % SPINNER_FRAMES.len()]);
                let text = msg2.lock().unwrap_or_else(|e| e.into_inner()).clone();
                // \r moves to column 0, \x1b[K clears to end of line
                let _ = write!(std::io::stderr(), "\r{frame}  {text}\x1b[K");
                let _ = std::io::stderr().flush();
                idx += 1;
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
        });
        Self {
            message: msg,
            handle: Some(handle),
            stop,
        }
    }

    fn set_message(&self, message: &str) {
        *self.message.lock().unwrap_or_else(|e| e.into_inner()) = message.to_string();
    }

    fn finish(mut self, final_line: &str) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let _ = write!(std::io::stderr(), "\r{final_line}\x1b[K\n");
        let _ = std::io::stderr().flush();
    }

    fn clear(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let _ = write!(std::io::stderr(), "\r\x1b[K");
        let _ = std::io::stderr().flush();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        // Don't join here — just signal stop. If someone forgot to call
        // finish/clear, the thread will notice on its next tick.
    }
}

fn format_stage_label(node_id: &str, agent_name: &str) -> String {
    if agent_name.is_empty() {
        node_id.to_string()
    } else {
        format!("{node_id} ({agent_name})")
    }
}

// ---------------------------------------------------------------------------
// ProgressEventEmitter
// ---------------------------------------------------------------------------

struct StageState {
    spinner: Option<Spinner>,
    started_at: Instant,
    agent_name: String,
}

pub struct ProgressEventEmitter {
    state: Mutex<HashMap<String, StageState>>,
}

impl Default for ProgressEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressEventEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

#[allow(clippy::print_stderr)]
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
                ..
            } => {
                if stage_index == 0 {
                    return;
                }
                let spinner = Spinner::new(node_id);
                state.insert(
                    node_id.clone(),
                    StageState {
                        spinner: Some(spinner),
                        started_at: Instant::now(),
                        agent_name: String::new(),
                    },
                );
            }

            PipelineEvent::StageInput {
                ref node_id,
                ref agent_name,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.agent_name = agent_name.clone();
                    if let Some(ref spinner) = s.spinner {
                        spinner.set_message(&format_stage_label(node_id, agent_name));
                    }
                }
            }

            PipelineEvent::StageOutput { .. } => {}

            PipelineEvent::StageCompleted { ref node_id, .. } => {
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let time_str = color("2", &format!("{elapsed:.1}s"));
                    let label = format_stage_label(node_id, &s.agent_name);
                    let line = format!("✅ {label}  {time_str}");

                    if let Some(spinner) = s.spinner {
                        spinner.finish(&line);
                    } else {
                        eprintln!("{line}");
                    }
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
                    let label = format_stage_label(node_id, &s.agent_name);
                    let line = format!("❌ {label}  {time_str}");

                    if let Some(spinner) = s.spinner {
                        spinner.finish(&line);
                    } else {
                        eprintln!("{line}");
                    }

                    eprintln!("{}", color("31", &format!("   Error: {reason}")));
                }
            }

            PipelineEvent::StageRetrying {
                ref node_id,
                attempt,
                max_attempts,
                ..
            } => {
                if let Some(s) = state.get(node_id) {
                    let label = format_stage_label(node_id, &s.agent_name);
                    if let Some(ref spinner) = s.spinner {
                        spinner
                            .set_message(&format!("{label} retrying ({attempt}/{max_attempts})…"));
                    }
                }
            }

            PipelineEvent::ParallelStarted { ref node_id } => {
                eprintln!("{}", color("2", &format!("║ parallel started: {node_id}")));
            }
            PipelineEvent::ParallelCompleted { ref node_id } => {
                eprintln!(
                    "{}",
                    color("2", &format!("║ parallel completed: {node_id}"))
                );
            }
            PipelineEvent::ParallelBranchStarted {
                ref node_id,
                branch_index,
            } => {
                eprintln!(
                    "{}",
                    color("2", &format!("║ branch {branch_index} started: {node_id}"))
                );
            }
            PipelineEvent::ParallelBranchCompleted {
                ref node_id,
                branch_index,
            } => {
                eprintln!(
                    "{}",
                    color(
                        "2",
                        &format!("║ branch {branch_index} completed: {node_id}")
                    )
                );
            }
            PipelineEvent::ParallelBranchFailed {
                ref node_id,
                branch_index,
                ref reason,
            } => {
                eprintln!(
                    "{}",
                    color(
                        "2",
                        &format!("║ branch {branch_index} failed: {node_id}: {reason}")
                    )
                );
            }

            PipelineEvent::InterviewQuestionAsked { ref node_id, .. } => {
                // Stop the spinner so dialoguer gets a clean terminal.
                if let Some(s) = state.get_mut(node_id)
                    && let Some(spinner) = s.spinner.take()
                {
                    spinner.clear();
                }
            }
            PipelineEvent::InterviewAnswerReceived { ref node_id, .. } => {
                // Print the ✅ completion line. The state entry is removed
                // so the subsequent StageCompleted is a no-op.
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let time_str = color("2", &format!("{elapsed:.1}s"));
                    let label = format_stage_label(node_id, &s.agent_name);
                    eprintln!("✅ {label}  {time_str}");
                }
            }
            PipelineEvent::InterviewTimedOut { ref node_id, .. } => {
                if let Some(s) = state.remove(node_id) {
                    let elapsed = s.started_at.elapsed().as_secs_f64();
                    let time_str = color("2", &format!("{elapsed:.1}s"));
                    let label = format_stage_label(node_id, &s.agent_name);
                    eprintln!("⏰ {label}  {time_str}");
                }
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

impl Default for VerboseEventEmitter {
    fn default() -> Self {
        Self::new()
    }
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
                ..
            } => {
                if stage_index == 0 {
                    return;
                }
                state.stages.insert(node_id.clone(), Instant::now());
                let bold_name = color("1", node_id);
                eprintln!("├─ {bold_name}");
            }

            PipelineEvent::StageInput {
                ref node_id,
                ref input,
                ref agent_name,
                ..
            } => {
                let preview = truncate_to(input, 100);
                let label_agent = color("2", "Agent:");
                eprintln!("│  {label_agent} {agent_name}");
                let _ = node_id;
                let label_prompt = color("2", "Prompt:");
                eprintln!("│  {label_prompt} {preview}");
            }

            PipelineEvent::StageSessionEvent { .. } => {}

            PipelineEvent::StageOutput { ref output, .. } => {
                let preview = truncate_to(output, 100);
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

            PipelineEvent::InterviewQuestionAsked { ref node_id, .. } => {
                eprintln!("│  ❔ waiting for human input at {node_id}…");
            }
            PipelineEvent::InterviewAnswerReceived { .. } => {}
            PipelineEvent::InterviewTimedOut { ref node_id, .. } => {
                eprintln!("│  ⏱ timed out waiting for input at {node_id}");
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

impl Default for PlainEventEmitter {
    fn default() -> Self {
        Self::new()
    }
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

            PipelineEvent::StageInput {
                ref node_id,
                ref agent_name,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.agent_name = agent_name.clone();
                }
            }

            PipelineEvent::StageOutput {
                ref node_id,
                ref output,
                ..
            } => {
                if let Some(s) = state.get_mut(node_id) {
                    s.response_preview = single_line_preview(output, 60);
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
