//! Pipeline execution events (§9.6).
//!
//! Defines the [`PipelineEvent`] enum covering five event categories:
//! pipeline lifecycle, stage lifecycle, parallel execution, interviews,
//! and checkpoints. The [`EventEmitter`] trait provides a synchronous
//! emission interface; [`NoOpEmitter`] is a default that discards events.

use crate::types::Outcome;

/// An event emitted during pipeline execution.
///
/// Events are grouped into five categories per §9.6:
/// - **Pipeline**: lifecycle of the overall pipeline run
/// - **Stage**: lifecycle of individual node executions
/// - **Parallel**: parallel fan-out/join events
/// - **Interview**: human-in-the-loop events
/// - **Checkpoint**: state persistence events
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    // -- Pipeline lifecycle --
    /// The pipeline has started execution.
    PipelineStarted {
        /// Name of the pipeline graph.
        pipeline_name: String,
    },
    /// The pipeline completed successfully.
    PipelineCompleted {
        /// Name of the pipeline graph.
        pipeline_name: String,
        /// Final outcome of the pipeline.
        outcome: Outcome,
    },
    /// The pipeline failed.
    PipelineFailed {
        /// Name of the pipeline graph.
        pipeline_name: String,
        /// Reason for failure.
        reason: String,
    },

    // -- Stage lifecycle --
    /// A stage (node) has started execution.
    StageStarted {
        /// Node ID being executed.
        node_id: String,
        /// Zero-based index of this stage in the traversal.
        stage_index: usize,
    },
    /// A stage completed successfully.
    StageCompleted {
        /// Node ID that completed.
        node_id: String,
        /// Zero-based index of this stage.
        stage_index: usize,
        /// Outcome of the stage execution.
        outcome: Outcome,
    },
    /// A stage failed.
    StageFailed {
        /// Node ID that failed.
        node_id: String,
        /// Zero-based index of this stage.
        stage_index: usize,
        /// Reason for the failure.
        reason: String,
    },
    /// A stage is being retried.
    StageRetrying {
        /// Node ID being retried.
        node_id: String,
        /// Zero-based index of this stage.
        stage_index: usize,
        /// Current attempt number (1-based).
        attempt: u32,
        /// Maximum number of attempts allowed.
        max_attempts: u32,
    },

    // -- Parallel --
    /// A parallel fan-out has started.
    ParallelStarted {
        /// Node ID of the parallel node.
        node_id: String,
    },
    /// A parallel branch has started.
    ParallelBranchStarted {
        /// Node ID of the parallel node.
        node_id: String,
        /// Index of the branch.
        branch_index: usize,
    },
    /// A parallel branch has completed.
    ParallelBranchCompleted {
        /// Node ID of the parallel node.
        node_id: String,
        /// Index of the branch.
        branch_index: usize,
    },
    /// A parallel branch has failed.
    ParallelBranchFailed {
        /// Node ID of the parallel node.
        node_id: String,
        /// Index of the branch.
        branch_index: usize,
        /// Reason for the failure.
        reason: String,
    },
    /// A parallel fan-out has completed (all branches joined).
    ParallelCompleted {
        /// Node ID of the parallel node.
        node_id: String,
    },

    // -- Interview --
    /// A question has been asked to a human.
    InterviewQuestionAsked {
        /// Node ID of the interview node.
        node_id: String,
    },
    /// A human has answered a question.
    InterviewAnswerReceived {
        /// Node ID of the interview node.
        node_id: String,
    },
    /// An interview timed out waiting for an answer.
    InterviewTimedOut {
        /// Node ID of the interview node.
        node_id: String,
    },

    // -- Checkpoint --
    /// A checkpoint has been saved.
    CheckpointSaved {
        /// Node ID at checkpoint time.
        node_id: String,
    },
}

/// Trait for receiving pipeline events.
///
/// Emission is synchronous and should be non-blocking. Implementations
/// that need to do async work should buffer events for later processing.
pub trait EventEmitter: Send + Sync {
    /// Emit a pipeline event.
    fn emit(&self, event: PipelineEvent);
}

/// A no-op event emitter that discards all events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpEmitter;

impl EventEmitter for NoOpEmitter {
    fn emit(&self, _event: PipelineEvent) {
        // Discard
    }
}

/// An event emitter that collects all events in-memory.
///
/// Useful for testing and post-run analysis.
pub struct CollectingEmitter {
    events: std::sync::Mutex<Vec<PipelineEvent>>,
}

impl CollectingEmitter {
    /// Create a new collecting emitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Return a clone of all collected events.
    #[must_use]
    pub fn events(&self) -> Vec<PipelineEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Return the number of collected events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Check if no events have been collected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for CollectingEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CollectingEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollectingEmitter")
            .field("count", &self.len())
            .finish()
    }
}

impl EventEmitter for CollectingEmitter {
    fn emit(&self, event: PipelineEvent) {
        let mut events = self
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        events.push(event);
    }
}

/// An event emitter that delegates to a callback function.
///
/// The callback receives each event synchronously and should be
/// non-blocking.
pub struct ObserverEmitter {
    callback: Box<dyn Fn(&PipelineEvent) + Send + Sync>,
}

impl ObserverEmitter {
    /// Create a new observer emitter with the given callback.
    pub fn new(callback: impl Fn(&PipelineEvent) + Send + Sync + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl std::fmt::Debug for ObserverEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserverEmitter").finish_non_exhaustive()
    }
}

impl EventEmitter for ObserverEmitter {
    fn emit(&self, event: PipelineEvent) {
        (self.callback)(&event);
    }
}

/// An event emitter that broadcasts to multiple inner emitters.
///
/// Useful for combining collecting + observer + other emitters.
pub struct BroadcastEmitter {
    emitters: Vec<Box<dyn EventEmitter>>,
}

impl BroadcastEmitter {
    /// Create a new broadcast emitter from a list of emitters.
    #[must_use]
    pub fn new(emitters: Vec<Box<dyn EventEmitter>>) -> Self {
        Self { emitters }
    }
}

impl std::fmt::Debug for BroadcastEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastEmitter")
            .field("count", &self.emitters.len())
            .finish()
    }
}

impl EventEmitter for BroadcastEmitter {
    fn emit(&self, event: PipelineEvent) {
        for emitter in &self.emitters {
            emitter.emit(event.clone());
        }
    }
}
