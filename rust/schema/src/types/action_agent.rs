// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::agent::Agent;
use super::organization::Organization;
use super::person::Person;
use super::software_application::SoftwareApplication;

/// A human, organization, software application, or Stencila AI agent that performs, provides, or participates in an action.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum ActionAgent {
    /// A human actor.
    #[default]
    Person(Person),

    /// An organization responsible for the action.
    Organization(Organization),

    /// A software tool or application that performed or assisted the action.
    SoftwareApplication(SoftwareApplication),

    /// A Stencila AI agent definition that performed or directed the action.
    Agent(Agent),
}
