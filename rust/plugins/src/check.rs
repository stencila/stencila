use cli_utils::{message, Message};
use codec::schema::shortcuts;
use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};
use kernel::schema::{Node, Null};
use model::{
    schema::{InstructionInline, InstructionMessage, MessagePart},
    GenerateOptions, GenerateTask, Instruction,
};

use crate::Plugin;

/// Check a plugin
#[tracing::instrument]
pub async fn check(name: &str) -> Result<Message> {
    tracing::info!("Checking plugin `{name}`");

    let plugin = Plugin::read_manifest(name)?;

    // Start and stop the plugin using each of the transports that
    // its manifest says it supports
    for transport in plugin.transports.clone() {
        tracing::info!("Checking plugin `{name}` with transport `{transport}`");

        // Start a plugin instance using transport
        let mut instance = plugin.start(Some(transport)).await?;

        // Health check (errors if not implemented or fails)
        instance.health().await?;

        // Stop the plugin instance
        instance.stop().await?;
    }

    // Call methods that should be implemented by the plugin based on its
    // manifest. These calls will create a new instances of the plugin.

    // Codecs
    for codec in plugin.codecs() {
        tracing::info!("Checking plugin `{name}` codec `{}`", codec.name());

        // Create an article with a single paragraph (because it should be handled
        // by almost all codecs).
        use shortcuts::{art, p, t};
        let node = art([p([t("Hello world")])]);

        // Encode to a string
        let (content, ..) = codec.to_string(&node, None).await?;

        // Decode from string
        let (decoded, ..) = codec.from_str(&content, None).await?;

        // Check roundtrip conversion worked
        if decoded != node {
            tracing::error!("Roundtrip encode-decode failed");
        }
    }

    // Kernels
    for kernel in plugin.kernels() {
        tracing::info!("Checking plugin `{name}` kernel `{}`", kernel.name());

        // Start an instance of the kernel
        let mut instance = kernel.create_instance()?;
        instance.start_here().await?;

        // Call methods on the instance. The return value will vary between
        // implementations so those are not checked.
        instance.info().await?;
        instance.packages().await?;
        instance.execute("code").await?;
        instance.evaluate("code").await?;
        instance.list().await?;
        instance.set("var", &Node::Null(Null)).await?;
        instance.get("var").await?;
        instance.remove("var").await?;

        // Stop the kernel instance
        instance.stop().await?;
    }

    // Assistants
    for assistant in plugin.assistants() {
        tracing::info!("Checking plugin `{name}` assistant `{}`", assistant.name());

        // Create a task for the assistant
        let instruction = Instruction::from(InstructionInline {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text("Say the word \"Hello\".".into())],
                ..Default::default()
            }],
            ..Default::default()
        });
        let task = GenerateTask::new(instruction, None);

        // Get the assistant to perform the task. Return value is not
        // checked since that will depend upon implementation
        assistant
            .perform_task(&task, &GenerateOptions::default())
            .await?;
    }

    Ok(message!(
        "💯 Successfully checked plugin `{}` version `{}`",
        plugin.name,
        plugin.version
    ))
}

/// Check a plugin
#[derive(Debug, Default, Args)]
pub struct CheckArgs {
    /// The name of the plugin to install
    pub name: String,
}

impl CheckArgs {
    pub async fn run(self) -> Result<Message> {
        check(&self.name).await
    }
}
