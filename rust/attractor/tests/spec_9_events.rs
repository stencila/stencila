//! Tests for pipeline events (§9.6).
//!
//! Covers all event variants, the collecting emitter, observer emitter,
//! and broadcast emitter.

use std::sync::{Arc, Mutex};

use stencila_attractor::events::{
    BroadcastEmitter, CollectingEmitter, EventEmitter, NoOpEmitter, ObserverEmitter, PipelineEvent,
};
use stencila_attractor::types::Outcome;

// ===========================================================================
// §9.6 — Event variants
// ===========================================================================

#[test]
fn event_pipeline_started_clone() {
    let event = PipelineEvent::PipelineStarted {
        pipeline_name: "test".into(),
    };
    let cloned = event.clone();
    assert!(format!("{cloned:?}").contains("test"));
}

#[test]
fn event_all_categories_exist() {
    // Verify all five categories have at least one variant constructible.
    let _pipeline = PipelineEvent::PipelineStarted {
        pipeline_name: "p".into(),
    };
    let _stage = PipelineEvent::StageStarted {
        node_id: "n".into(),
        stage_index: 0,
    };
    let _parallel = PipelineEvent::ParallelStarted {
        node_id: "n".into(),
    };
    let _interview = PipelineEvent::InterviewQuestionAsked {
        node_id: "n".into(),
    };
    let _checkpoint = PipelineEvent::CheckpointSaved {
        node_id: "n".into(),
    };
}

#[test]
fn event_stage_completed_contains_outcome() {
    let event = PipelineEvent::StageCompleted {
        node_id: "task1".into(),
        stage_index: 2,
        outcome: Outcome::success(),
    };
    let debug = format!("{event:?}");
    assert!(debug.contains("task1"));
    assert!(debug.contains("Success"));
}

#[test]
fn event_stage_retrying_fields() {
    let event = PipelineEvent::StageRetrying {
        node_id: "n".into(),
        stage_index: 0,
        attempt: 2,
        max_attempts: 5,
    };
    let debug = format!("{event:?}");
    assert!(debug.contains("attempt: 2"));
    assert!(debug.contains("max_attempts: 5"));
}

#[test]
fn event_parallel_branch_failed_has_reason() {
    let event = PipelineEvent::ParallelBranchFailed {
        node_id: "par".into(),
        branch_index: 1,
        reason: "timeout".into(),
    };
    let debug = format!("{event:?}");
    assert!(debug.contains("timeout"));
}

#[test]
fn event_pipeline_completed_has_outcome() {
    let event = PipelineEvent::PipelineCompleted {
        pipeline_name: "pipe".into(),
        outcome: Outcome::fail("done"),
    };
    let debug = format!("{event:?}");
    assert!(debug.contains("Fail"));
}

// ===========================================================================
// NoOpEmitter
// ===========================================================================

#[test]
fn noop_emitter_discards() {
    let emitter = NoOpEmitter;
    // Should not panic
    emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: "test".into(),
    });
    emitter.emit(PipelineEvent::StageStarted {
        node_id: "n".into(),
        stage_index: 0,
    });
}

// ===========================================================================
// CollectingEmitter
// ===========================================================================

#[test]
fn collecting_emitter_captures_events() {
    let emitter = CollectingEmitter::new();
    assert!(emitter.is_empty());

    emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: "p".into(),
    });
    emitter.emit(PipelineEvent::StageStarted {
        node_id: "n1".into(),
        stage_index: 0,
    });
    emitter.emit(PipelineEvent::StageCompleted {
        node_id: "n1".into(),
        stage_index: 0,
        outcome: Outcome::success(),
    });

    assert_eq!(emitter.len(), 3);
    assert!(!emitter.is_empty());

    let events = emitter.events();
    assert_eq!(events.len(), 3);
}

#[test]
fn collecting_emitter_default() {
    let emitter = CollectingEmitter::default();
    assert!(emitter.is_empty());
}

#[test]
fn collecting_emitter_debug() {
    let emitter = CollectingEmitter::new();
    emitter.emit(PipelineEvent::CheckpointSaved {
        node_id: "n".into(),
    });
    let debug = format!("{emitter:?}");
    assert!(debug.contains("1"));
}

// ===========================================================================
// ObserverEmitter
// ===========================================================================

#[test]
fn observer_emitter_calls_callback() {
    let count = Arc::new(Mutex::new(0u32));
    let count_clone = Arc::clone(&count);

    let emitter = ObserverEmitter::new(move |_event| {
        let mut c = count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *c += 1;
    });

    emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: "test".into(),
    });
    emitter.emit(PipelineEvent::PipelineCompleted {
        pipeline_name: "test".into(),
        outcome: Outcome::success(),
    });

    let final_count = *count
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(final_count, 2);
}

#[test]
fn observer_emitter_receives_event_data() {
    let names = Arc::new(Mutex::new(Vec::new()));
    let names_clone = Arc::clone(&names);

    let emitter = ObserverEmitter::new(move |event| {
        if let PipelineEvent::StageStarted { node_id, .. } = event {
            names_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(node_id.clone());
        }
    });

    emitter.emit(PipelineEvent::StageStarted {
        node_id: "task1".into(),
        stage_index: 0,
    });
    emitter.emit(PipelineEvent::StageStarted {
        node_id: "task2".into(),
        stage_index: 1,
    });

    let captured = names
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(captured.as_slice(), &["task1", "task2"]);
}

#[test]
fn observer_emitter_debug() {
    let emitter = ObserverEmitter::new(|_| {});
    let debug = format!("{emitter:?}");
    assert!(debug.contains("ObserverEmitter"));
}

// ===========================================================================
// BroadcastEmitter
// ===========================================================================

#[test]
fn broadcast_emitter_sends_to_all() {
    let c1 = CollectingEmitter::new();
    let c2 = CollectingEmitter::new();

    // We need to use Arc to share the collecting emitters
    let c1 = Arc::new(c1);
    let c2 = Arc::new(c2);

    // Clone Arcs for checking after broadcast
    let c1_check = Arc::clone(&c1);
    let c2_check = Arc::clone(&c2);

    // Use observer emitters that delegate to the collectors
    let o1 = ObserverEmitter::new(move |event| c1.emit(event.clone()));
    let o2 = ObserverEmitter::new(move |event| c2.emit(event.clone()));

    let broadcast = BroadcastEmitter::new(vec![Box::new(o1), Box::new(o2)]);

    broadcast.emit(PipelineEvent::PipelineStarted {
        pipeline_name: "test".into(),
    });

    assert_eq!(c1_check.len(), 1);
    assert_eq!(c2_check.len(), 1);
}

#[test]
fn broadcast_emitter_empty_is_ok() {
    let broadcast = BroadcastEmitter::new(vec![]);
    // Should not panic
    broadcast.emit(PipelineEvent::PipelineStarted {
        pipeline_name: "test".into(),
    });
}

#[test]
fn broadcast_emitter_debug() {
    let broadcast = BroadcastEmitter::new(vec![Box::new(NoOpEmitter), Box::new(NoOpEmitter)]);
    let debug = format!("{broadcast:?}");
    assert!(debug.contains("2"));
}
