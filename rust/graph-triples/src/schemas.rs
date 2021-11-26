use crate::{Relation, Resource};
use schemars::schema_for;
use serde_json::json;

/// Get JSON Schema definitions for types in this crate
pub fn schemas() -> serde_json::Value {
    json!([
        schema_for!(Resource),
        schema_for!(Relation),
        json!({
            "$id": "Triple",
            "title": "Triple",
            "description": "A subject-relation-object triple",
            "type" : "array",
            "items": [
                {
                    "tsType": "Resource"
                },
                {
                    "tsType": "Relation"
                },
                {
                    "tsType": "Resource"
                }
            ],
            "minItems": 3,
            "maxItems": 3
        })
    ])
}
