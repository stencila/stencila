// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::grant::Grant;
use super::monetary_grant::MonetaryGrant;

/// [`Grant`] or [`MonetaryGrant`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum GrantOrMonetaryGrant {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}
