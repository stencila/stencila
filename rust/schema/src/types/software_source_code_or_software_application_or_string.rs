//! Generated file, do not edit

use crate::prelude::*;

use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::string::String;

/// [`SoftwareSourceCode`] or [`SoftwareApplication`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum SoftwareSourceCodeOrSoftwareApplicationOrString {
    SoftwareSourceCode(SoftwareSourceCode),
    SoftwareApplication(SoftwareApplication),
    String(String),
}
