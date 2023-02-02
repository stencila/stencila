//! Generated file, do not edit

use crate::prelude::*;

use super::grant::Grant;
use super::monetary_grant::MonetaryGrant;

/// [`Grant`] or [`MonetaryGrant`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]

pub enum GrantOrMonetaryGrant {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}
