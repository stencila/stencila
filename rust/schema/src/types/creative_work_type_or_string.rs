use crate::prelude::*;

use super::creative_work_type::CreativeWorkType;
use super::string::String;

/// [`CreativeWorkType`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum CreativeWorkTypeOrString {
    CreativeWorkType(CreativeWorkType),
    String(String),
}
