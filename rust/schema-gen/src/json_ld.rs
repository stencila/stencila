//! Generation of a JSON-LD context from Stencila Schema

use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use common::{
    eyre::Result,
    itertools::Itertools,
    serde_json::{self, json, Value},
};

use crate::{
    schema::{Items, Schema, Type},
    schemas::Schemas,
};

impl Schemas {
    /// Generate JSON-LD context for the schemas
    pub async fn json_ld(&self) -> Result<()> {
        eprintln!("Generating JSON-LD");
        
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../json/");

        // For each property determine its `domainIncludes` (type it exists on)
        // and `rangeIncludes` (types it can have).
        let mut domains = HashMap::new();
        for schema in self.schemas.values() {
            for (property_name, property) in &schema.properties {
                if property.is_inherited {
                    continue;
                }
                let Some(jid) = &schema.jid  else {
                    continue;
                };

                let item = json!({ "@id": jid });
                domains
                    .entry(property_name)
                    .and_modify(|entry: &mut Vec<Value>| entry.push(item.clone()))
                    .or_insert_with(|| vec![item]);
            }
        }

        // Generate a schema for each schema
        for (title, schema) in self.schemas.iter() {
            let path = dir.join(format!("{title}.jsonld"));
            let mut file = File::create(path)?;

            let context = json!({
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "schema": "https://schema.org/",
                "stencila": "https://stencila.dev/",
            });

            let mut class = json!({
                "@id": schema.jid,
                "@type": "rdfs:Class",
                "rdfs:label": schema.title,
                "rdfs:comment": schema.description,
            });

            if let Some(extends) = schema.extends.first() {
                class["rdfs:subClassOf"] = json!({ "@id": extends });
            }

            let mut graph = vec![class];

            for (property_name, property) in &schema.properties {
                if property_name == "type" || property_name == "id" {
                    continue;
                }

                let ranges = ranges(property)
                    .iter()
                    .map(|id| json!({ "@id": id }))
                    .collect_vec();

                let mut prop = json!({
                    "@id": property.jid,
                    "@type": "rdfs:Property",
                    "rdfs:label": property_name,
                    "rdfs:comment": property.description,
                });

                let domains = &domains[property_name];
                prop["schema:domainIncludes"] = if domains.len() == 1 {
                    json!(domains[0])
                } else {
                    json!(domains)
                };

                if !ranges.is_empty() {
                    prop["schema:rangeIncludes"] = if ranges.len() == 1 {
                        json!(ranges[0])
                    } else {
                        json!(ranges)
                    };
                }

                graph.push(prop)
            }

            let jsonld = json!({
                "@context": context,
                "@graph": graph
            });

            let jsonld = serde_json::to_string_pretty(&jsonld)?;

            file.write_all(jsonld.as_bytes())?;
        }

        fn ranges(property: &Schema) -> Vec<String> {
            let mut ids = vec![];

            if let Some(r#type) = &property.r#type {
                match r#type {
                    Type::Boolean => ids.push("schema:Boolean".to_string()),
                    Type::Integer | Type::Number => ids.push("schema:Number".to_string()),
                    Type::String => ids.push("schema:Text".to_string()),
                    Type::Array => {
                        if let Some(items) = &property.items {
                            match items {
                                Items::Ref(inner) => ids.push(format!("stencila:{}", inner.r#ref)),
                                Items::Type(inner) => ids.append(&mut ranges(&Schema {
                                    r#type: Some(inner.r#type.clone()),
                                    ..Default::default()
                                })),
                                Items::AnyOf(inner) => ids.append(&mut ranges(&Schema {
                                    any_of: Some(inner.any_of.clone()),
                                    ..Default::default()
                                })),
                                Items::List(inner) => ids.append(&mut ranges(&Schema {
                                    any_of: Some(inner.clone()),
                                    ..Default::default()
                                })),
                            }
                        }
                    }
                    _ => {}
                }
            } else if let Some(any_of) = &property.any_of {
                ids.append(&mut any_of.iter().flat_map(ranges).collect_vec())
            }

            ids
        }

        Ok(())
    }
}
