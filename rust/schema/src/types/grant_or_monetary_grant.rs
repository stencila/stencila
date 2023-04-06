use crate::prelude::*;

use super::grant::Grant;
use super::monetary_grant::MonetaryGrant;

/// [`Grant`] or [`MonetaryGrant`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum GrantOrMonetaryGrant {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}
