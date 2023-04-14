use crate::prelude::*;

use super::postal_address::PostalAddress;
use super::string::String;

/// [`PostalAddress`] or [`String`]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum PostalAddressOrString {
    PostalAddress(PostalAddress),
    String(String),
}
