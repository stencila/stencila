use std::sync::Arc;

use serde::{Deserialize, Serialize};

use async_trait::async_trait;
use eyre::{Result, bail};
use inflector::Inflector;
use stencila_model::{Model, ModelAvailability, ModelIO, ModelOutput, ModelTask, ModelType};
use tokio::sync::Mutex;

use crate::{Plugin, PluginEnabled, PluginInstance, PluginStatus, plugins};

/// A model provided by a plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginModel {
    /// The id of the model
    id: String,

    /// The name of the model
    ///
    /// Will be extracted from the id if not supplied
    name: Option<String>,

    /// A description of the model
    description: Option<String>,

    /// The input types that the model supports
    #[serde(default)]
    inputs: Vec<ModelIO>,

    /// The output types that the model supports
    #[serde(default)]
    outputs: Vec<ModelIO>,

    /// The plugin that provides this model
    ///
    /// Used to be able to create a plugin instance, which in
    /// turn is used to create a model instance.
    #[serde(skip)]
    plugin: Option<Plugin>,

    /// The plugin instance for this model. Used to avoid starting
    /// a new instance for each call to the model.
    ///
    /// This needs to be a `Arc<Mutex>` because the `perform_task` method is async
    /// but is not `&mut self`. As such, this is needed for "interior mutability" across
    /// calls to that method.
    #[serde(skip)]
    plugin_instance: Arc<Mutex<Option<PluginInstance>>>,
}

impl PluginModel {
    /// Bind a plugin to this model so that it can be started (by starting the plugin first)
    pub fn bind(&mut self, plugin: &Plugin) {
        self.plugin = Some(plugin.clone());
    }
}

#[async_trait]
impl Model for PluginModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn r#type(&self) -> ModelType {
        match &self.plugin {
            Some(plugin) => {
                let mut name = plugin.name.clone();
                if plugin.linked {
                    name += " (linked)";
                }
                ModelType::Plugin(name)
            }
            None => ModelType::Plugin("unknown".to_string()),
        }
    }

    fn availability(&self) -> ModelAvailability {
        match &self.plugin {
            Some(plugin) => match plugin.availability() {
                (
                    PluginStatus::InstalledLatest(..) | PluginStatus::InstalledOutdated(..),
                    PluginEnabled::Yes,
                ) => ModelAvailability::Available,

                (
                    PluginStatus::InstalledLatest(..) | PluginStatus::InstalledOutdated(..),
                    PluginEnabled::No,
                ) => ModelAvailability::Disabled,

                (PluginStatus::Installable, _) => ModelAvailability::Installable,

                _ => ModelAvailability::Unavailable,
            },
            None => ModelAvailability::Unavailable,
        }
    }

    fn name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            let id = self.id.clone();
            let name = id
                .rsplit_once('/')
                .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
                .unwrap_or(&id);
            name.to_title_case()
        })
    }

    fn version(&self) -> String {
        self.plugin
            .as_ref()
            .map(|plugin| plugin.version.to_string())
            .unwrap_or_default()
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        if self.inputs.is_empty() {
            &[ModelIO::Text]
        } else {
            &self.inputs
        }
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        if self.outputs.is_empty() {
            &[ModelIO::Text]
        } else {
            &self.outputs
        }
    }

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
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
                guard.as_mut().expect("assigned on line above")
            }
        };

        // Call the plugin's `model_perform_task`` method
        #[derive(Serialize)]
        struct Params<'node> {
            model: String,
            task: &'node ModelTask,
        }
        let result: ModelOutput = instance
            .call(
                "model_perform_task",
                Params {
                    model: self.id.clone(),
                    task,
                },
            )
            .await?;

        Ok(result)
    }
}

/// List all the models provided by plugins
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    Ok(plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.models())
        .collect())
}
