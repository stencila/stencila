use std::sync::Arc;

use assistant::{
    format::Format, Assistant, AssistantIO, AssistantType, GenerateOptions, GenerateOutput,
    GenerateTask,
};
use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    tokio::sync::Mutex,
};

use crate::{plugins, Plugin, PluginInstance};

/// A assistant provided by a plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct PluginAssistant {
    /// The id of the assistant
    id: String,

    /// The input types that the assistant supports
    #[serde(default)]
    inputs: Vec<AssistantIO>,

    /// The output types that the assistant supports
    #[serde(default)]
    outputs: Vec<AssistantIO>,

    /// The plugin that provides this assistant
    ///
    /// Used to be able to create a plugin instance, which in
    /// turn is used to create a assistant instance.
    #[serde(skip)]
    plugin: Option<Plugin>,

    /// The plugin instance for this assistant. Used to avoid starting
    /// a new instance for each call to the assistant.
    ///
    /// This needs to be a `Arc<Mutex>` because the `perform_task` method async
    /// but is not `&mut self`. So, this is needed for "interior mutability" across
    /// calls to that method.
    #[serde(skip)]
    plugin_instance: Arc<Mutex<Option<PluginInstance>>>,
}

impl PluginAssistant {
    /// Bind a plugin to this assistant so that it can be started (by starting the plugin first)
    pub fn bind(&mut self, plugin: &Plugin) {
        self.plugin = Some(plugin.clone());
    }
}

#[async_trait]
impl Assistant for PluginAssistant {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn r#type(&self) -> AssistantType {
        match &self.plugin {
            Some(plugin) => {
                let mut name = plugin.name.clone();
                if plugin.linked {
                    name += " (linked)";
                }
                AssistantType::Plugin(name)
            }
            None => AssistantType::Plugin("unknown".to_string()),
        }
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        if self.inputs.is_empty() {
            &[AssistantIO::Text]
        } else {
            &self.inputs
        }
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        if self.outputs.is_empty() {
            &[AssistantIO::Text]
        } else {
            &self.outputs
        }
    }

    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        // Create the plugin instance if necessary
        let mut guard = self.plugin_instance.lock().await;
        let instance = match &mut *guard {
            Some(instance) => instance,
            None => {
                let Some(plugin) = self.plugin.as_ref() else {
                    bail!("Not bound yet")
                };

                let inst = plugin.start(None).await?;
                *guard = Some(inst);
                guard.as_mut().unwrap()
            }
        };

        // Call the plugin method
        #[derive(Serialize)]
        #[serde(crate = "common::serde")]
        struct Params {
            assistant: String,
            task: GenerateTask,
            options: GenerateOptions,
        }
        let output: GenerateOutput = instance
            .call(
                "assistant_execute",
                Params {
                    assistant: self.id(),
                    task: task.clone(),
                    options: options.clone(),
                },
            )
            .await?;

        // Post process the output
        let format = if output.format == Format::Unknown {
            task.format().clone()
        } else {
            output.format.clone()
        };
        GenerateOutput::from_plugin(output, self, &format, task.instruction(), options).await
    }
}

/// List all the assistants provided by plugins
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    Ok(plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.assistants())
        .collect())
}
