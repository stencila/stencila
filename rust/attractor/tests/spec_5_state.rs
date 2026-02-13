//! Tests for state management (§5.3–5.5).
//!
//! Covers artifact store (§5.5), fidelity resolution (§5.4),
//! thread ID resolution (§5.4), and checkpoint resume (§5.3).

mod common;

use std::collections::HashSet;

use stencila_attractor::artifact::ArtifactStore;
use stencila_attractor::checkpoint::Checkpoint;
use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorResult;
use stencila_attractor::fidelity::{resolve_fidelity, resolve_thread_id};
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::resume::resume_from_checkpoint;
use stencila_attractor::types::FidelityMode;

use common::make_tempdir;

// ===========================================================================
// §5.5 — Artifact store
// ===========================================================================

#[test]
fn artifact_store_in_memory() -> AttractorResult<()> {
    let store = ArtifactStore::new(None);
    let data = b"hello world";

    let info = store.store("art1", "my artifact", data)?;
    assert_eq!(info.id, "art1");
    assert_eq!(info.name, "my artifact");
    assert_eq!(info.size_bytes, 11);
    assert!(!info.is_file_backed);

    assert!(store.has("art1"));
    assert!(!store.has("art2"));

    let retrieved = store.retrieve("art1")?;
    assert_eq!(retrieved, data);

    Ok(())
}

#[test]
fn artifact_store_list() -> AttractorResult<()> {
    let store = ArtifactStore::new(None);
    store.store("a", "Alpha", b"data1")?;
    store.store("b", "Beta", b"data2")?;

    let list = store.list();
    assert_eq!(list.len(), 2);

    let ids: HashSet<String> = list.iter().map(|i| i.id.clone()).collect();
    assert!(ids.contains("a"));
    assert!(ids.contains("b"));

    Ok(())
}

#[test]
fn artifact_store_remove() -> AttractorResult<()> {
    let store = ArtifactStore::new(None);
    store.store("a", "Alpha", b"data")?;
    assert!(store.has("a"));

    store.remove("a");
    assert!(!store.has("a"));

    Ok(())
}

#[test]
fn artifact_store_clear() -> AttractorResult<()> {
    let store = ArtifactStore::new(None);
    store.store("a", "Alpha", b"data1")?;
    store.store("b", "Beta", b"data2")?;

    store.clear();
    assert!(!store.has("a"));
    assert!(!store.has("b"));
    assert!(store.list().is_empty());

    Ok(())
}

#[test]
fn artifact_store_retrieve_missing() {
    let store = ArtifactStore::new(None);
    let result = store.retrieve("nonexistent");
    assert!(result.is_err());
}

#[test]
fn artifact_store_file_backed() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let store = ArtifactStore::new(Some(tmp.path().to_path_buf()));

    // Create data larger than 100KB threshold
    let large_data = vec![0u8; 150 * 1024];
    let info = store.store("big", "Large artifact", &large_data)?;

    assert!(info.is_file_backed);
    assert_eq!(info.size_bytes, 150 * 1024);

    // Verify file exists on disk
    let artifact_path = tmp.path().join("artifacts/big.json");
    assert!(artifact_path.exists());

    // Retrieve should return same data
    let retrieved = store.retrieve("big")?;
    assert_eq!(retrieved.len(), 150 * 1024);

    // Remove should delete file
    store.remove("big");
    assert!(!artifact_path.exists());

    Ok(())
}

#[test]
fn artifact_store_small_stays_in_memory() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let store = ArtifactStore::new(Some(tmp.path().to_path_buf()));

    let small_data = b"small data";
    let info = store.store("small", "Small", small_data)?;

    assert!(!info.is_file_backed);
    // No file on disk
    assert!(!tmp.path().join("artifacts/small.json").exists());

    Ok(())
}

#[test]
fn artifact_store_base_dir() {
    let store_none = ArtifactStore::new(None);
    assert!(store_none.base_dir().is_none());

    let store_some = ArtifactStore::new(Some("/tmp/test".into()));
    assert!(store_some.base_dir().is_some());
}

#[test]
fn artifact_store_overwrite() -> AttractorResult<()> {
    let store = ArtifactStore::new(None);
    store.store("a", "First", b"data1")?;
    store.store("a", "Second", b"data2")?;

    let retrieved = store.retrieve("a")?;
    assert_eq!(retrieved, b"data2");

    let list = store.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Second");

    Ok(())
}

/// Overwriting a file-backed artifact with a small (in-memory) artifact
/// must clean up the old file on disk.
#[test]
fn artifact_store_overwrite_cleans_old_file() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let store = ArtifactStore::new(Some(tmp.path().to_path_buf()));

    // Store a large artifact (file-backed)
    let large_data = vec![0u8; 200 * 1024]; // 200KB → file-backed
    let info = store.store("doc", "Large", &large_data)?;
    assert!(info.is_file_backed);
    let file_path = tmp.path().join("artifacts/doc.json");
    assert!(file_path.exists());

    // Overwrite with a small artifact (in-memory)
    let info2 = store.store("doc", "Small", b"tiny")?;
    assert!(!info2.is_file_backed);

    // Old file should be cleaned up
    assert!(!file_path.exists());

    // Data should be the new value
    let retrieved = store.retrieve("doc")?;
    assert_eq!(retrieved, b"tiny");

    Ok(())
}

// ===========================================================================
// §5.4 — Fidelity resolution
// ===========================================================================

#[test]
fn fidelity_default_is_compact() {
    let node = Node::new("n");
    let g = Graph::new("test");
    let mode = resolve_fidelity(&node, None, &g);
    assert_eq!(mode, FidelityMode::Compact);
}

#[test]
fn fidelity_node_level() {
    let mut node = Node::new("n");
    node.attrs
        .insert("fidelity".into(), AttrValue::from("full"));
    let g = Graph::new("test");
    let mode = resolve_fidelity(&node, None, &g);
    assert_eq!(mode, FidelityMode::Full);
}

#[test]
fn fidelity_edge_overrides_node() {
    let mut node = Node::new("n");
    node.attrs
        .insert("fidelity".into(), AttrValue::from("compact"));

    let mut edge = Edge::new("prev", "n");
    edge.attrs
        .insert("fidelity".into(), AttrValue::from("truncate"));

    let g = Graph::new("test");
    let mode = resolve_fidelity(&node, Some(&edge), &g);
    assert_eq!(mode, FidelityMode::Truncate);
}

#[test]
fn fidelity_graph_level_default() {
    let node = Node::new("n");
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("default_fidelity".into(), AttrValue::from("summary:high"));
    let mode = resolve_fidelity(&node, None, &g);
    assert_eq!(mode, FidelityMode::SummaryHigh);
}

#[test]
fn fidelity_node_overrides_graph() {
    let mut node = Node::new("n");
    node.attrs
        .insert("fidelity".into(), AttrValue::from("full"));
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("default_fidelity".into(), AttrValue::from("compact"));
    let mode = resolve_fidelity(&node, None, &g);
    assert_eq!(mode, FidelityMode::Full);
}

#[test]
fn fidelity_invalid_value_falls_through() {
    let mut node = Node::new("n");
    node.attrs
        .insert("fidelity".into(), AttrValue::from("invalid_mode"));
    let g = Graph::new("test");
    // Invalid mode on node → falls through to default
    let mode = resolve_fidelity(&node, None, &g);
    assert_eq!(mode, FidelityMode::Compact);
}

#[test]
fn fidelity_summary_variants() {
    for (input, expected) in [
        ("summary:low", FidelityMode::SummaryLow),
        ("summary:medium", FidelityMode::SummaryMedium),
        ("summary:high", FidelityMode::SummaryHigh),
    ] {
        let mut node = Node::new("n");
        node.attrs.insert("fidelity".into(), AttrValue::from(input));
        let g = Graph::new("test");
        let mode = resolve_fidelity(&node, None, &g);
        assert_eq!(mode, expected, "failed for {input}");
    }
}

// ===========================================================================
// §5.4 — Thread ID resolution
// ===========================================================================

#[test]
fn thread_id_fallback_to_previous_node() {
    let node = Node::new("n");
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, None, &g, "prev_node");
    assert_eq!(tid, "prev_node");
}

#[test]
fn thread_id_from_node_attr() {
    let mut node = Node::new("n");
    node.attrs
        .insert("thread_id".into(), AttrValue::from("my_thread"));
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, None, &g, "prev");
    assert_eq!(tid, "my_thread");
}

#[test]
fn thread_id_from_edge_attr() {
    let node = Node::new("n");
    let mut edge = Edge::new("prev", "n");
    edge.attrs
        .insert("thread_id".into(), AttrValue::from("edge_thread"));
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, Some(&edge), &g, "prev");
    assert_eq!(tid, "edge_thread");
}

#[test]
fn thread_id_node_overrides_edge() {
    let mut node = Node::new("n");
    node.attrs
        .insert("thread_id".into(), AttrValue::from("node_thread"));
    let mut edge = Edge::new("prev", "n");
    edge.attrs
        .insert("thread_id".into(), AttrValue::from("edge_thread"));
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, Some(&edge), &g, "prev");
    assert_eq!(tid, "node_thread");
}

#[test]
fn thread_id_from_graph_default() {
    let node = Node::new("n");
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("default_thread_id".into(), AttrValue::from("global_thread"));
    let tid = resolve_thread_id(&node, None, &g, "prev");
    assert_eq!(tid, "global_thread");
}

#[test]
fn thread_id_from_class_attr() {
    let mut node = Node::new("n");
    node.attrs
        .insert("class".into(), AttrValue::from("review, test"));
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, None, &g, "prev");
    assert_eq!(tid, "review"); // first class
}

#[test]
fn thread_id_class_empty_falls_through() {
    let mut node = Node::new("n");
    node.attrs.insert("class".into(), AttrValue::from(""));
    let g = Graph::new("test");
    let tid = resolve_thread_id(&node, None, &g, "prev");
    assert_eq!(tid, "prev"); // falls through to previous node
}

// ===========================================================================
// §5.3 — Checkpoint resume
// ===========================================================================

#[test]
fn resume_restores_context() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let checkpoint_path = tmp.path().join("checkpoint.json");

    // Build a simple graph: start → middle → exit
    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);
    g.add_node(Node::new("middle"));
    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);
    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));

    // Create and save a checkpoint at "start"
    let ctx = Context::new();
    ctx.set("goal", serde_json::json!("test goal"));
    ctx.append_log("log entry 1");
    let checkpoint = Checkpoint::from_context(
        &ctx,
        "start",
        vec!["start".into()],
        indexmap::IndexMap::new(),
        indexmap::IndexMap::new(),
    );
    checkpoint.save(&checkpoint_path)?;

    // Resume from checkpoint
    let state = resume_from_checkpoint(&checkpoint_path, &g)?;

    // Context should have the saved value
    assert_eq!(
        state.context.get("goal"),
        Some(serde_json::json!("test goal"))
    );
    // Logs should be restored
    assert_eq!(state.context.logs(), vec!["log entry 1"]);
    // Completed nodes
    assert!(state.completed_nodes_ordered.contains(&"start".to_string()));
    // Next node should be "middle" (first outgoing edge from "start")
    assert_eq!(state.next_node_id, "middle");
    // No fidelity degradation (start has no fidelity attr)
    assert!(!state.degrade_fidelity);

    Ok(())
}

#[test]
fn resume_fidelity_degradation() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let checkpoint_path = tmp.path().join("checkpoint.json");

    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut middle = Node::new("middle");
    middle
        .attrs
        .insert("fidelity".into(), AttrValue::from("full"));
    g.add_node(middle);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);
    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));

    // Checkpoint at "middle" which has full fidelity
    let ctx = Context::new();
    let checkpoint = Checkpoint::from_context(
        &ctx,
        "middle",
        vec!["start".into(), "middle".into()],
        indexmap::IndexMap::new(),
        indexmap::IndexMap::new(),
    );
    checkpoint.save(&checkpoint_path)?;

    let state = resume_from_checkpoint(&checkpoint_path, &g)?;

    // Should degrade fidelity because previous node was "full"
    assert!(state.degrade_fidelity);
    assert_eq!(state.next_node_id, "exit");

    Ok(())
}

#[test]
fn resume_retry_counters_restored() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let checkpoint_path = tmp.path().join("checkpoint.json");

    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);
    g.add_node(Node::new("middle"));
    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);
    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));

    let ctx = Context::new();
    let mut retries = indexmap::IndexMap::new();
    retries.insert("middle".to_string(), 3u32);
    let checkpoint = Checkpoint::from_context(
        &ctx,
        "start",
        vec!["start".into()],
        indexmap::IndexMap::new(),
        retries,
    );
    checkpoint.save(&checkpoint_path)?;

    let state = resume_from_checkpoint(&checkpoint_path, &g)?;

    let retry_count = state.context.get("internal.retry_count.middle");
    assert_eq!(retry_count, Some(serde_json::json!(3)));

    Ok(())
}

#[test]
fn resume_at_exit_node() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let checkpoint_path = tmp.path().join("checkpoint.json");

    let mut g = Graph::new("test");
    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    let ctx = Context::new();
    let checkpoint = Checkpoint::from_context(
        &ctx,
        "exit",
        vec!["exit".into()],
        indexmap::IndexMap::new(),
        indexmap::IndexMap::new(),
    );
    checkpoint.save(&checkpoint_path)?;

    let state = resume_from_checkpoint(&checkpoint_path, &g)?;

    // Should resume at exit itself (no more nodes)
    assert_eq!(state.next_node_id, "exit");

    Ok(())
}
