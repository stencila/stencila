use serde::Serialize;
use strum::{EnumString, EnumVariantNames, ToString};
#[derive(Debug, EnumString, EnumVariantNames, PartialEq, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Method {
    Decode,
    Encode,
    Execute,
}
