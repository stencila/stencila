//! Phase 2 / Slice 3 tests: CreateSessionOptions and convenience API
//!
//! Verifies that `CreateSessionOptions` has the correct fields with the
//! expected types, that `create_session_with_options` exists, and that the
//! existing convenience functions (`create_session`, `create_session_with_overrides`)
//! still compile with their original signatures.

#![allow(clippy::result_large_err)]

use stencila_agents::convenience::{
    CreateSessionOptions, SessionOverrides, create_session, create_session_with_options,
    create_session_with_overrides,
};
use stencila_agents::store::SessionPersistence;
use stencila_agents::types::Turn;

// ===========================================================================
// AC-1: CreateSessionOptions struct has all required fields
// ===========================================================================

#[test]
fn create_session_options_has_interviewer_field() {
    let opts = CreateSessionOptions::default();
    // interviewer: Option<Arc<dyn Interviewer>> — should be None by default
    assert!(
        opts.interviewer.is_none(),
        "interviewer should default to None"
    );
}

#[test]
fn create_session_options_has_overrides_field() {
    let opts = CreateSessionOptions::default();
    assert!(opts.overrides.is_none(), "overrides should default to None");
}

#[test]
fn create_session_options_has_persistence_field() {
    let opts = CreateSessionOptions::default();
    assert!(
        opts.persistence.is_none(),
        "persistence should default to None"
    );
}

#[test]
fn create_session_options_has_session_id_field() {
    let opts = CreateSessionOptions::default();
    assert!(
        opts.session_id.is_none(),
        "session_id should default to None"
    );
}

#[test]
fn create_session_options_has_history_field() {
    let opts = CreateSessionOptions::default();
    assert!(opts.history.is_none(), "history should default to None");
}

#[test]
fn create_session_options_has_total_turns_field() {
    let opts = CreateSessionOptions::default();
    assert_eq!(opts.total_turns, 0, "total_turns should default to 0");
}

// ===========================================================================
// AC-1: Field types are correct (constructible with expected types)
// ===========================================================================

#[test]
fn create_session_options_accepts_persistence_type() {
    let opts = CreateSessionOptions {
        persistence: Some(SessionPersistence::Persistent),
        ..Default::default()
    };
    assert!(opts.persistence.is_some());
}

#[test]
fn create_session_options_accepts_session_id_string() {
    let opts = CreateSessionOptions {
        session_id: Some("custom-session-id".to_string()),
        ..Default::default()
    };
    assert_eq!(opts.session_id.as_deref(), Some("custom-session-id"));
}

#[test]
fn create_session_options_accepts_history_vec_turn() {
    let history = vec![Turn::user("hello"), Turn::assistant("world")];
    let opts = CreateSessionOptions {
        history: Some(history.clone()),
        ..Default::default()
    };
    assert_eq!(opts.history.as_ref().map(|h| h.len()), Some(2));
}

#[test]
fn create_session_options_accepts_overrides() {
    let overrides = SessionOverrides {
        model: Some("test-model".to_string()),
        ..Default::default()
    };
    let opts = CreateSessionOptions {
        overrides: Some(overrides),
        ..Default::default()
    };
    assert!(opts.overrides.is_some());
}

#[test]
fn create_session_options_accepts_total_turns() {
    let opts = CreateSessionOptions {
        total_turns: 42,
        ..Default::default()
    };
    assert_eq!(opts.total_turns, 42);
}

// ===========================================================================
// AC-2: create_session_with_options function exists and has correct signature
// ===========================================================================

#[test]
fn create_session_with_options_is_async_fn() {
    // Verify the function exists and returns a Future.
    // We construct default options but do NOT await — we only need to check
    // that the call expression compiles with the expected (name, options) signature.
    let _future = create_session_with_options("test-agent", CreateSessionOptions::default());
    // Dropping the future without awaiting is intentional — we're testing the
    // API shape, not executing the I/O-heavy session creation.
}

// ===========================================================================
// AC-3: create_session(name) still compiles with original signature
// ===========================================================================

#[test]
fn create_session_original_signature_compiles() {
    // create_session takes a single &str and returns a Future.
    let _future = create_session("test-agent");
}

// ===========================================================================
// AC-4: create_session_with_overrides still compiles with original signature
// ===========================================================================

#[test]
fn create_session_with_overrides_original_signature_compiles() {
    let overrides = SessionOverrides::default();
    // create_session_with_overrides(name, interviewer, overrides)
    let _future = create_session_with_overrides("test-agent", None, &overrides);
}

// ===========================================================================
// AC-5: Default options produce equivalent behavior to create_session
// ===========================================================================

#[test]
fn default_options_all_fields_are_none_or_zero() {
    let opts = CreateSessionOptions::default();
    assert!(opts.interviewer.is_none());
    assert!(opts.overrides.is_none());
    assert!(opts.persistence.is_none());
    assert!(opts.session_id.is_none());
    assert!(opts.history.is_none());
    assert_eq!(opts.total_turns, 0);
}
