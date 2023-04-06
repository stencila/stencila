use crate::prelude::*;

use super::postal_address::PostalAddress;
use super::string::String;

/// [`PostalAddress`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum PostalAddressOrString {
    PostalAddress(PostalAddress),
    String(String),
}
