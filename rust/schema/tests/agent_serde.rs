//! Deserialization tests for Agent schema changes: plural models/providers with
//! backward-compatible singular aliases, and the new modelSize property.
//!
//! These tests verify acceptance criteria for Phase 2 / Slice 1.

use eyre::Result;

use stencila_schema::Agent;

// --- AC7: singular `model` alias deserializes to `models` vec ---

#[test]
fn agent_singular_model_deserializes_to_models_vec() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
model: sonnet
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.models,
        Some(vec!["sonnet".to_string()]),
        "singular `model: sonnet` should deserialize to models vec"
    );
    Ok(())
}

// --- AC8: plural `models` array deserializes correctly ---

#[test]
fn agent_plural_models_deserializes_correctly() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
models:
  - sonnet
  - gpt
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.models,
        Some(vec!["sonnet".to_string(), "gpt".to_string()]),
        "plural `models: [sonnet, gpt]` should deserialize correctly"
    );
    Ok(())
}

// --- AC9: singular `provider` alias deserializes to `providers` vec ---

#[test]
fn agent_singular_provider_deserializes_to_providers_vec() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
provider: anthropic
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.providers,
        Some(vec!["anthropic".to_string()]),
        "singular `provider: anthropic` should deserialize to providers vec"
    );
    Ok(())
}

// --- AC10: `modelSize` property deserializes correctly ---

#[test]
fn agent_model_size_deserializes() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
modelSize: medium
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.model_size,
        Some("medium".to_string()),
        "modelSize: medium should deserialize to model_size"
    );
    Ok(())
}

// --- AC6: Generated Rust struct has correct field types ---

#[test]
fn agent_struct_has_models_field_type() {
    // Construct an Agent with models set to verify the type is Option<Vec<String>>
    let agent = Agent {
        models: Some(vec!["sonnet".to_string(), "gpt".to_string()]),
        ..Agent::new("A test agent".to_string(), "test-agent".to_string())
    };
    assert_eq!(agent.models.as_ref().map(Vec::len), Some(2));
}

#[test]
fn agent_struct_has_providers_field_type() {
    // Construct an Agent with providers set to verify the type is Option<Vec<String>>
    let agent = Agent {
        providers: Some(vec!["anthropic".to_string()]),
        ..Agent::new("A test agent".to_string(), "test-agent".to_string())
    };
    assert_eq!(agent.providers.as_ref().map(Vec::len), Some(1));
}

#[test]
fn agent_struct_has_model_size_field_type() {
    // Construct an Agent with model_size set to verify the type is Option<String>
    let agent = Agent {
        model_size: Some("large".to_string()),
        ..Agent::new("A test agent".to_string(), "test-agent".to_string())
    };
    assert_eq!(agent.model_size, Some("large".to_string()));
}

// --- AC8 extended: single-element `models` array ---

#[test]
fn agent_models_single_element_array() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
models:
  - opus
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.models,
        Some(vec!["opus".to_string()]),
        "single-element models array should work"
    );
    Ok(())
}

// --- AC9 extended: plural `providers` array ---

#[test]
fn agent_plural_providers_deserializes_correctly() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
providers:
  - anthropic
  - openai
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.providers,
        Some(vec!["anthropic".to_string(), "openai".to_string()]),
        "plural `providers: [anthropic, openai]` should deserialize correctly"
    );
    Ok(())
}

// --- Combined: all new fields together ---

#[test]
fn agent_all_new_fields_together() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
models:
  - sonnet
  - gpt
providers:
  - anthropic
  - openai
modelSize: medium
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.models,
        Some(vec!["sonnet".to_string(), "gpt".to_string()])
    );
    assert_eq!(
        agent.providers,
        Some(vec!["anthropic".to_string(), "openai".to_string()])
    );
    assert_eq!(agent.model_size, Some("medium".to_string()));
    Ok(())
}

// --- model_size kebab-case alias ---

#[test]
fn agent_model_size_snake_case_alias() -> Result<()> {
    let yaml = r"
type: Agent
name: test-agent
description: A test agent
model_size: small
content: []
";
    let agent: Agent = serde_yaml::from_str(yaml)?;
    assert_eq!(
        agent.model_size,
        Some("small".to_string()),
        "model_size (snake_case alias) should deserialize to model_size"
    );
    Ok(())
}
