use std::str::FromStr;

use serde_json::json;

use stencila_codec::{
    NodeType,
    stencila_format::Format,
    stencila_schema::{
        Agent, Article, Chat, CodeLocation, CompilationMessage, MessageLevel, Node, Prompt, Skill,
        shortcuts::t,
    },
};

use super::{Context, decode_blocks, decode_inlines};

/// Parse any YAML frontmatter
///
/// Aims to be as robust as possible to syntax and semantic errors in YAML
/// while maintaining at least the type of the node.
pub fn frontmatter(yaml: &str, node_type: Option<NodeType>) -> (Node, Vec<CompilationMessage>) {
    let mut messages = Vec::new();

    // Deserialize YAML to a value, and add `type` properties if necessary
    let (mut value, node_type) = match serde_yaml::from_str(yaml) {
        Ok(serde_json::Value::Object(mut value)) => {
            let node_type = node_type.or_else(|| {
                value
                    .get("type")
                    .and_then(|value| value.as_str())
                    .and_then(|value| NodeType::from_str(value).ok())
            });

            let node_type = if let Some(node_type) = node_type {
                // Ensure that the right type and `content` is present for types that
                // require it, so that `serde_json::from_value` succeeds
                if matches!(
                    node_type,
                    NodeType::Article | NodeType::Prompt | NodeType::Chat | NodeType::Skill
                ) && value.get("content").is_none()
                {
                    value.insert("type".into(), json!(node_type.to_string()));
                    value.insert("content".to_string(), json!([]));
                } else if node_type == NodeType::Agent {
                    // Agent has optional content — only inject `type`, not `content`
                    value.insert("type".into(), json!(node_type.to_string()));
                }
                node_type
            } else {
                // Assume an article
                value.insert("type".into(), json!("Article"));
                value.insert("content".into(), json!([]));

                NodeType::Article
            };

            (json!(value), node_type)
        }
        res => {
            match res {
                Ok(..) => {
                    messages.push(CompilationMessage::new(
                        MessageLevel::Error,
                        "Expected a YAML key:value map, got a different type".to_string(),
                    ));
                }
                Err(error) => {
                    let code_location = error.location().map(|loc| CodeLocation {
                        start_line: Some(loc.line() as u64),
                        start_column: Some(loc.column().saturating_sub(1) as u64),
                        ..Default::default()
                    });
                    messages.push(CompilationMessage {
                        level: MessageLevel::Error,
                        message: error.to_string(),
                        code_location,
                        ..Default::default()
                    });
                }
            }

            let node_type = if let Some(node_type) = node_type {
                node_type
            } else if yaml.contains("type: Agent") {
                NodeType::Agent
            } else if yaml.contains("type: Prompt") {
                NodeType::Prompt
            } else if yaml.contains("type: Skill") {
                NodeType::Skill
            } else if yaml.contains("type: Chat") {
                NodeType::Chat
            } else {
                NodeType::Article
            };

            (
                json!({"type": node_type.to_string(), "content": []}),
                node_type,
            )
        }
    };

    // Parse title and abstract as Markdown (need to do here before deserializing to node
    // and remove from value so does not cause an error when deserializing)
    let context = &mut Context::new(Format::Markdown);
    let (title, abs) = if let Some(object) = value.as_object_mut() {
        let title = object
            .remove("title")
            .and_then(|value| value.as_str().map(String::from))
            .map(|title| decode_inlines(&title, context));
        let abs = object
            .remove("abstract")
            .and_then(|value| value.as_str().map(String::from))
            .map(|abs| decode_blocks(&abs, context));
        (title, abs)
    } else {
        (None, None)
    };

    // Prompts require a title but the above stanza removes it, so add a placeholder
    // (replaced below) to ensure value gets deserialized as a prompt
    if let Some("Prompt") = value.get("type").and_then(|typ| typ.as_str()) {
        value["title"] = json!([]);
    }

    // For Skills and Agents, hoist `metadata` entries to top level so they
    // map to CreativeWork fields (per Agent Skills Spec §metadata)
    if matches!(node_type, NodeType::Skill | NodeType::Agent)
        && let Some(object) = value.as_object_mut()
        && let Some(serde_json::Value::Object(meta)) = object.remove("metadata")
    {
        for (key, val) in meta {
            object.entry(&key).or_insert(val);
        }
    }

    // Deserialize value to the node type
    let mut node = match node_type {
        NodeType::Agent => serde_json::from_value::<Agent>(value).map_or_else(
            |error| {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("{error} in YAML frontmatter"),
                ));
                Node::Agent(Agent::default())
            },
            Node::Agent,
        ),
        NodeType::Prompt => serde_json::from_value::<Prompt>(value).map_or_else(
            |error| {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("{error} in YAML frontmatter"),
                ));
                Node::Prompt(Prompt::default())
            },
            Node::Prompt,
        ),
        NodeType::Skill => serde_json::from_value::<Skill>(value).map_or_else(
            |error| {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("{error} in YAML frontmatter"),
                ));
                Node::Skill(Skill::default())
            },
            Node::Skill,
        ),
        NodeType::Chat => serde_json::from_value::<Chat>(value).map_or_else(
            |error| {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("{error} in YAML frontmatter"),
                ));
                Node::Chat(Chat::default())
            },
            Node::Chat,
        ),
        _ => serde_json::from_value::<Article>(value).map_or_else(
            |error| {
                messages.push(CompilationMessage::new(
                    MessageLevel::Error,
                    format!("{error} in YAML frontmatter"),
                ));
                Node::Article(Article::default())
            },
            Node::Article,
        ),
    };

    // Set title and abstract for node types that have them
    match &mut node {
        Node::Article(article) => {
            article.title = title;
            article.r#abstract = abs;
        }
        Node::Prompt(prompt) => {
            prompt.title = title.unwrap_or_else(|| vec![t("Untitled")]);
            prompt.options.r#abstract = abs;
        }
        Node::Agent(agent) => {
            agent.options.title = title;
            agent.options.r#abstract = abs;
        }
        Node::Skill(skill) => {
            skill.options.title = title;
            skill.options.r#abstract = abs;
        }
        Node::Chat(chat) => {
            chat.title = title;
            chat.options.r#abstract = abs;
        }
        _ => {}
    }

    (node, messages)
}
