use serde_json::{Map, Value};
use stencila_codec::Losses;

pub(crate) fn oxa_type_str(obj: &Map<String, Value>) -> &str {
    obj.get("type").and_then(|v| v.as_str()).unwrap_or("")
}

pub(crate) fn record_classes_loss(obj: &Map<String, Value>, losses: &mut Losses) {
    if obj
        .get("classes")
        .and_then(|v| v.as_array())
        .is_some_and(|a| !a.is_empty())
    {
        losses.add("oxa_classes_dropped");
    }
}
