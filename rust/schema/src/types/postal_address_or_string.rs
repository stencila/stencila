use crate::prelude::*;

use super::postal_address::PostalAddress;
use super::string::String;

/// [`PostalAddress`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write)]
#[serde(untagged, crate = "common::serde")]

pub enum PostalAddressOrString {
    PostalAddress(PostalAddress),
    String(String),
}
