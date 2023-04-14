use crate::prelude::*;

use super::image_object::ImageObject;
use super::string::String;

/// [`ImageObject`] or [`String`]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum ImageObjectOrString {
    ImageObject(ImageObject),
    String(String),
}
