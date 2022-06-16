//! Type definitions for Stencila Cloud API types
//! 
//! To avoid drift, prefer to only only add properties that are needed here to these structs.

use common::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ApiToken {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct User {
    pub id: String,
    pub short_name: String,
    pub long_name: String,
    pub email: String,
}
