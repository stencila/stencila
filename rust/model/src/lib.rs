use common::{
    async_trait::async_trait,
    clap::{self, ValueEnum},
    eyre::Result,
    inflector::Inflector,
    serde::{Deserialize, Serialize},
    strum::Display,
};

use schema::{
    AuthorRole, AuthorRoleAuthor, AuthorRoleName, InstructionMessage, MessagePart, MessageRole,
    Organization, PersonOrOrganization, SoftwareApplication, SoftwareApplicationOptions,
    StringOrNumber, Timestamp,
};

// Export crates for the convenience of dependant crates
pub use common;
pub use format;
pub use schema;
pub use secrets;

mod output;
mod task;
pub use output::{ModelOutput, ModelOutputKind};
pub use task::{ModelTask, ModelTaskKind};

/// The type of provider of a model
///
/// This ordering here is important as it is used when
/// selecting a model to execute a task.
#[derive(Display, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum ModelType {
    Builtin,
    Local,
    Router,
    Remote,
    Proxied,
    Plugin(String),
}

/// The availability of a model on the current machine
#[derive(Display, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[strum(serialize_all = "lowercase")]
pub enum ModelAvailability {
    /// Available on this machine
    Available,
    /// Requires API key or token
    #[strum(to_string = "requires api key")]
    RequiresKey,
    /// Requires installation
    Installable,
    /// Not available on this machine
    Unavailable,
    /// Available on this machine but disabled
    Disabled,
}

/// The type of model input or output
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum, Display, Deserialize, Serialize,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum ModelIO {
    #[default]
    Text,
    Image,
    Audio,
    Video,
}

/// Specifications for a model
///
/// Currently used only for outputs and display.
#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde", rename_all = "camelCase")]
pub struct ModelSpecification {
    id: String,
    provider: String,
    name: String,
    version: String,
    r#type: ModelType,
    availability: ModelAvailability,
}

impl From<&dyn Model> for ModelSpecification {
    fn from(model: &dyn Model) -> Self {
        Self {
            id: model.id(),
            provider: model.provider(),
            name: model.name(),
            version: model.version(),
            r#type: model.r#type(),
            availability: model.availability(),
        }
    }
}

/// A generative model
///
/// Provides a common, shared interface for the various generative models
/// and APIs used. Model implementations should override `supports_task` and other methods.
#[async_trait]
pub trait Model: Sync + Send {
    /// Get the id of the model
    ///
    /// The id should be unique amongst models.
    /// The id should follow the pattern <PUBLISHER>/<MODEL>.
    fn id(&self) -> String;

    /// Get the type of the model
    fn r#type(&self) -> ModelType {
        ModelType::Builtin
    }

    /// Get the availability of the model
    fn availability(&self) -> ModelAvailability {
        ModelAvailability::Available
    }

    /// Is the model currently available?
    fn is_available(&self) -> bool {
        matches!(self.availability(), ModelAvailability::Available)
    }

    /// Get the name of the provider of the model
    ///
    /// This default implementation returns the title cased name
    /// before the first forward slash in the name. Derived models
    /// should override if necessary.
    fn provider(&self) -> String {
        let name = self.id();
        let provider = name
            .split_once('/')
            .map(|(publisher, ..)| publisher)
            .unwrap_or(&name);
        provider.to_title_case()
    }

    /// Get the name of the model
    ///
    /// This default implementation returns the title cased name
    /// after the last forward slash but before the first dash in the name.
    /// Derived models should override if necessary.
    fn name(&self) -> String {
        let name = self.id();
        let name = name
            .rsplit_once('/')
            .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
            .unwrap_or(&name);
        name.to_title_case()
    }

    /// Get the version of the model
    ///
    /// This default implementation returns the version after the
    /// first dash in the name. Derived models should override if necessary.
    fn version(&self) -> String {
        let name = self.id();
        let version = name
            .split_once('-')
            .map(|(.., version)| version)
            .unwrap_or_default();
        version.to_string()
    }

    /// A description of the model in Markdown
    fn description(&self) -> Option<String> {
        None
    }

    /// Create an `AuthorRole` node for this model
    fn to_author_role(&self, role_name: AuthorRoleName) -> AuthorRole {
        let mut role = AuthorRole::new(
            AuthorRoleAuthor::SoftwareApplication(self.to_software_application()),
            role_name,
        );
        role.last_modified = Some(Timestamp::now());
        role
    }

    /// Create a `SoftwareApplication` node identifying this model
    ///
    /// Intended for usage in the `authors` property of inner document
    /// nodes where it is desirable to have minimal identifying information
    /// only.
    fn to_software_application(&self) -> SoftwareApplication {
        SoftwareApplication {
            id: Some(self.id()),
            name: self.name(),
            version: Some(StringOrNumber::String(self.version())),
            ..Default::default()
        }
    }

    /// Create a `SoftwareApplication` node representing this model
    ///
    /// Intended for usage in the `authors` or `contributors` property
    /// of the root `CreativeWork`.
    fn to_software_application_complete(&self) -> SoftwareApplication {
        SoftwareApplication {
            id: Some(self.id()),
            name: self.name(),
            version: Some(StringOrNumber::String(self.version())),
            options: Box::new(SoftwareApplicationOptions {
                publisher: Some(PersonOrOrganization::Organization(Organization {
                    name: Some(self.provider()),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Get the context length of the model
    ///
    /// Used by custom models to dynamically adjust the content of prompts
    /// based on the context length of the underlying model being used.
    fn context_length(&self) -> usize {
        0
    }

    /// Does the model support a specific task
    ///
    /// This default implementation is based solely on whether the models
    /// supports the input/output combination of the task. Overrides may
    /// add other criteria such as the type of the task's instruction.
    fn supports_task(&self, task: &ModelTask) -> bool {
        self.supported_task_kinds().contains(&task.kind)
    }

    /// Get a list of task kinds this model supports
    fn supported_task_kinds(&self) -> &[ModelTaskKind] {
        &[ModelTaskKind::MessageGeneration]
    }

    /// Get a list of input types this model supports
    fn supported_inputs(&self) -> &[ModelIO] {
        &[]
    }

    /// Get a list of output types this model supports
    fn supported_outputs(&self) -> &[ModelIO] {
        &[]
    }

    /// Perform a generation task
    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput>;
}

/// Generate a test task which has system, user and model messages
///
/// Used for tests of implementations of the `Model` trait to check that
/// the system prompt, and each user and model message, are being sent to
/// and processed by the model.
#[allow(unused)]
pub fn test_task_repeat_word() -> ModelTask {
    ModelTask {
        messages: vec![
            InstructionMessage {
                role: Some(MessageRole::System),
                parts: vec![MessagePart::Text("When asked to repeat a word, you should repeat it in ALL CAPS. Do not provide any other notes, explanation or content.".into())],
                ..Default::default()
            },
            InstructionMessage {
                role: Some(MessageRole::User),
                parts: vec![MessagePart::Text("Say the word \"Hello\".".into())],
                ..Default::default()
            },
            InstructionMessage {
                role: Some(MessageRole::Model),
                parts: vec![MessagePart::Text("Hello".into())],
                ..Default::default()
            },
            InstructionMessage {
                role: Some(MessageRole::User),
                parts: vec![MessagePart::Text("Repeat the word.".into())],
                ..Default::default()
            },
        ],
        temperature: Some(0.),
        ..Default::default()
    }
}
