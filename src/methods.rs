use serde::Serialize;
use strum::{EnumString, EnumVariantNames};
#[derive(Debug, EnumString, EnumVariantNames, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Method {
    Decode,
    Encode,
}
