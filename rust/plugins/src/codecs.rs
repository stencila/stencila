use std::sync::Arc;

use codec::{schema::Node, status::Status, Codec, EncodeInfo, EncodeOptions};
use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    tokio::sync::Mutex,
};

use crate::{plugins, Plugin, PluginInstance};

/// A codec provided by a plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct PluginCodec {
    /// The name of the codec
    name: String,

    /// The plugin that provides this codec
    ///
    /// Used to be able to create a plugin instance, which in
    /// turn is used to create a codec instance.
    #[serde(skip)]
    plugin: Option<Plugin>,

    /// The plugin instance for this codec. Used to avoid starting
    /// a new instance for each call to the codec.
    ///
    /// This needs to be a `Arc<Mutex>` because the `to_string`, `from_string` etc methods are async
    /// but not `&mut self`. As such, this is needed for "interior mutability" across
    /// calls to those methods.
    #[serde(skip)]
    plugin_instance: Arc<Mutex<Option<PluginInstance>>>,
}

impl PluginCodec {
    /// Bind a plugin to this codec so that it can be started (by starting the plugin first)
    pub fn bind(&mut self, plugin: &Plugin) {
        self.plugin = Some(plugin.clone());
    }
}

#[async_trait]
impl Codec for PluginCodec {
    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
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
                guard.as_mut().expect("should have been set above")
            }
        };

        // Call the plugin's to_string method
        #[derive(Serialize)]
        #[serde(crate = "common::serde")]
        struct Params<'node> {
            codec: String,
            node: &'node Node,
        }
        let result: String = instance
            .call(
                "codec_to_string",
                Params {
                    codec: self.name.clone(),
                    node,
                },
            )
            .await?;

        Ok((result, EncodeInfo::none()))
    }
}

/// List all the codecs provided by plugins
pub async fn list() -> Result<Vec<Box<dyn Codec>>> {
    Ok(plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.codecs())
        .collect())
}
