use pretty_assertions::assert_eq;
use stencila_codec::{
    Codec, DecodeOptions,
    eyre::Result,
    stencila_schema::{Node, NodeType},
};
use stencila_codec_markdown::MarkdownCodec;

/// Helper to decode a markdown string as a Workflow (by setting `node_type`)
async fn decode_workflow(md: &str) -> Result<Node> {
    let codec = MarkdownCodec {};
    let (node, ..) = codec
        .from_str(
            md,
            Some(DecodeOptions {
                node_type: Some(NodeType::Workflow),
                ..Default::default()
            }),
        )
        .await?;
    Ok(node)
}

/// Helper to encode a node back to markdown string
async fn encode_node(node: &Node) -> Result<String> {
    let codec = MarkdownCodec {};
    let (md, ..) = codec.to_string(node, None).await?;
    Ok(md)
}

/// Decode a full WORKFLOW.md with frontmatter + dot code block and verify all fields
#[tokio::test]
async fn decode_full_workflow() -> Result<()> {
    let md = r#"---
name: code-review
description: Implements, tests, and reviews code changes
goal: "Implement and validate a feature"
modelStylesheet: |
  box { model = "claude-sonnet-4-5" }
  .critical { reasoning_effort = "high" }
---

# Code Review Pipeline

This workflow plans, implements, and validates code with human review.

```dot
digraph code_review {
    rankdir=LR
    node [shape=box, timeout="900s"]

    start     [shape=Mdiamond]
    exit      [shape=Msquare]
    plan      [agent="code-planner", prompt="Plan the implementation for: $goal"]
    implement [agent="code-engineer", prompt="Implement the plan", goal_gate=true]
    validate  [agent="code-engineer", prompt="Run tests"]
    review    [shape=hexagon, label="Review Changes"]

    start -> plan -> implement -> validate -> review
    review -> exit      [label="[A] Approve"]
    review -> implement [label="[F] Fix"]
}
```

The pipeline uses a human gate at the review stage.
"#;

    let node = decode_workflow(md).await?;

    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(workflow.name, "code-review");
    assert_eq!(
        workflow.description,
        "Implements, tests, and reviews code changes"
    );
    assert_eq!(
        workflow.goal.as_deref(),
        Some("Implement and validate a feature")
    );

    // Pipeline should be extracted from the dot code block
    let pipeline = workflow.pipeline.as_deref().expect("pipeline should be Some");
    assert!(pipeline.contains("digraph code_review"));
    assert!(pipeline.contains("start -> plan -> implement -> validate -> review"));
    assert!(pipeline.contains(r#"agent="code-planner""#));

    // Content should be present (heading, paragraph, code block, trailing paragraph)
    assert!(workflow.content.is_some());
    let content = workflow.content.as_ref().unwrap();
    assert!(!content.is_empty());

    // modelStylesheet should be parsed from frontmatter
    let stylesheet = workflow
        .options
        .model_stylesheet
        .as_deref()
        .expect("modelStylesheet should be Some");
    assert!(stylesheet.contains("claude-sonnet-4-5"));
    assert!(stylesheet.contains("reasoning_effort"));

    // Frontmatter should be stored
    assert!(workflow.frontmatter.is_some());

    Ok(())
}

/// Encode a Workflow back to markdown and verify frontmatter + content are emitted
#[tokio::test]
async fn encode_workflow_roundtrip() -> Result<()> {
    let md = r#"---
name: code-review
description: Implements, tests, and reviews code changes
goal: "Implement and validate a feature"
---

# Code Review Pipeline

```dot
digraph code_review {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    start -> exit
}
```
"#;

    let node = decode_workflow(md).await?;
    let encoded = encode_node(&node).await?;

    // Encoded output should contain frontmatter delimiters
    assert!(encoded.contains("---"));

    // Should contain the workflow metadata
    assert!(encoded.contains("name: code-review"));
    assert!(encoded.contains("description: Implements, tests, and reviews code changes"));

    // Should contain the dot code block
    assert!(encoded.contains("```dot"));
    assert!(encoded.contains("digraph code_review"));

    // Should contain the heading
    assert!(encoded.contains("# Code Review Pipeline"));

    Ok(())
}

/// Pipeline extraction: first dot code block found; non-dot blocks are ignored
#[tokio::test]
async fn pipeline_extraction_first_dot_block() -> Result<()> {
    let md = r#"---
name: test-wf
description: test
---

```python
print("not a pipeline")
```

```dot
digraph first { a -> b }
```

```dot
digraph second { c -> d }
```
"#;

    let node = decode_workflow(md).await?;
    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    let pipeline = workflow.pipeline.as_deref().expect("pipeline should be Some");
    // Should extract the first dot block, not the second
    assert!(pipeline.contains("digraph first"));
    assert!(!pipeline.contains("digraph second"));

    Ok(())
}

/// Missing dot block → pipeline should be None
#[tokio::test]
async fn pipeline_none_when_no_dot_block() -> Result<()> {
    let md = r#"---
name: test-wf
description: test
---

# Just a heading

Some text but no dot code block.

```python
print("not dot")
```
"#;

    let node = decode_workflow(md).await?;
    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert!(workflow.pipeline.is_none());

    Ok(())
}

/// Frontmatter-only workflow (no body) — content should be None or empty, pipeline should be None
#[tokio::test]
async fn frontmatter_only_no_body() -> Result<()> {
    let md = r#"---
name: minimal-wf
description: A minimal workflow with no body
---
"#;

    let node = decode_workflow(md).await?;
    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(workflow.name, "minimal-wf");
    assert_eq!(workflow.description, "A minimal workflow with no body");
    assert!(workflow.pipeline.is_none());
    // Content should be None (no body blocks)
    assert!(workflow.content.is_none());

    Ok(())
}

/// Body with no dot block — content should be present but pipeline should be None
#[tokio::test]
async fn body_with_no_dot_block() -> Result<()> {
    let md = r#"---
name: doc-only
description: Workflow with documentation but no pipeline
---

# Documentation

This workflow has documentation but no pipeline definition yet.
"#;

    let node = decode_workflow(md).await?;
    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(workflow.name, "doc-only");
    assert!(workflow.content.is_some());
    assert!(!workflow.content.as_ref().unwrap().is_empty());
    assert!(workflow.pipeline.is_none());

    Ok(())
}

/// Type detection via frontmatter `type: Workflow` without explicit node_type hint
#[tokio::test]
async fn type_detection_from_frontmatter() -> Result<()> {
    let codec = MarkdownCodec {};
    let md = r#"---
type: Workflow
name: auto-detect
description: Should be detected as Workflow from type field
---

```dot
digraph auto { x -> y }
```
"#;

    // Decode WITHOUT specifying node_type — should still produce a Workflow
    let (node, ..) = codec.from_str(md, None).await?;

    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(workflow.name, "auto-detect");
    assert!(workflow.pipeline.is_some());
    assert!(
        workflow
            .pipeline
            .as_deref()
            .unwrap()
            .contains("digraph auto")
    );

    Ok(())
}

/// Goal and optional fields are properly decoded
#[tokio::test]
async fn optional_fields_decoded() -> Result<()> {
    let md = r#"---
name: full-options
description: Workflow with all optional fields
goal: "Ship the feature"
defaultMaxRetry: 10
retryTarget: plan
fallbackRetryTarget: start
defaultFidelity: "summary:medium"
---
"#;

    let node = decode_workflow(md).await?;
    let workflow = match &node {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(workflow.goal.as_deref(), Some("Ship the feature"));
    assert_eq!(workflow.options.default_max_retry, Some(10));
    assert_eq!(workflow.options.retry_target.as_deref(), Some("plan"));
    assert_eq!(
        workflow.options.fallback_retry_target.as_deref(),
        Some("start")
    );
    assert_eq!(
        workflow.options.default_fidelity.as_deref(),
        Some("summary:medium")
    );

    Ok(())
}

/// Decode then re-encode preserves essential content
#[tokio::test]
async fn decode_encode_roundtrip_preserves_content() -> Result<()> {
    let original = r#"---
name: roundtrip
description: Test round-trip fidelity
goal: "Test everything"
---

# Pipeline

```dot
digraph roundtrip {
    start [shape=Mdiamond]
    work  [agent="worker"]
    exit  [shape=Msquare]
    start -> work -> exit
}
```

## Notes

Some additional documentation.
"#;

    let node = decode_workflow(original).await?;
    let encoded = encode_node(&node).await?;

    // Re-decode and verify fields survive the roundtrip
    let node2 = decode_workflow(&encoded).await?;
    let w2 = match &node2 {
        Node::Workflow(w) => w,
        other => panic!("Expected Workflow, got {:?}", other),
    };

    assert_eq!(w2.name, "roundtrip");
    assert_eq!(w2.description, "Test round-trip fidelity");
    assert_eq!(w2.goal.as_deref(), Some("Test everything"));
    assert!(w2.pipeline.is_some());
    assert!(
        w2.pipeline
            .as_deref()
            .unwrap()
            .contains("digraph roundtrip")
    );

    Ok(())
}
