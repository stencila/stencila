use cli_utils::{message, Message};
use codec::schema::shortcuts;
use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};
use kernel::schema::{Node, Null};
use model::{
    schema::{InstructionMessage, MessagePart},
    ModelTask,
};

use crate::Plugin;

/// Check a plugin
#[tracing::instrument]
pub async fn check(
    name: &str,
    skip_codecs: bool,
    skip_kernels: bool,
    skip_models: bool,
) -> Result<Message> {
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
        if skip_codecs {
            tracing::warn!("Skipping plugin `{name}` codec `{}`", codec.name());
            continue;
        }

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
        if skip_kernels {
            tracing::warn!("Skipping plugin `{name}` kernel `{}`", kernel.name());
            continue;
        }

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

    // Models
    for model in plugin.models() {
        if skip_models {
            tracing::warn!("Skipping plugin `{name}` model `{}`", model.name());
            continue;
        }

        tracing::info!("Checking plugin `{name}` model `{}`", model.id());

        // Create a task for the model
        let task = ModelTask {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text("Say the word \"Hello\".".into())],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Get the model to perform the task. Return value is not
        // checked since that will depend upon implementation
        model.perform_task(&task).await?;
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

    /// Skip checking codecs
    #[arg(long)]
    pub skip_codecs: bool,

    /// Skip checking kernels
    #[arg(long)]
    pub skip_kernels: bool,

    /// Skip checking models
    #[arg(long)]
    pub skip_models: bool,
}

impl CheckArgs {
    pub async fn run(self) -> Result<Message> {
        check(
            &self.name,
            self.skip_codecs,
            self.skip_kernels,
            self.skip_models,
        )
        .await
    }
}
