//! Generated file, do not edit

use crate::prelude::*;

use super::image_object::ImageObject;
use super::string::String;

/// [`ImageObject`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum ImageObjectOrString {
    ImageObject(ImageObject),
    String(String),
}
