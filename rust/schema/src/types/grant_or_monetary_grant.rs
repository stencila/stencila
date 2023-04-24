use crate::prelude::*;

use super::grant::Grant;
use super::monetary_grant::MonetaryGrant;

/// [`Grant`] or [`MonetaryGrant`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum GrantOrMonetaryGrant {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}
