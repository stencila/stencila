//! Generation of a JSON-LD context from Stencila Schema

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use common::{
    eyre::Result,
    futures::future::try_join_all,
    glob::glob,
    indexmap::IndexMap,
    itertools::Itertools,
    serde_json::{self, json},
    tokio::fs::{remove_file, write},
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

        // Remove all existing *.jsonld files
        let futures = glob(&dir.join("*.jsonld").to_string_lossy())?
            .flatten()
            .map(|file| async { remove_file(file).await });
        try_join_all(futures).await?;

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

                domains
                    .entry(property_name)
                    .and_modify(|entry: &mut Vec<String>| entry.push(jid.clone()))
                    .or_insert_with(|| vec![jid.clone()]);
            }
        }

        // Generate a schema for each schema
        let mut overall = BTreeMap::new();
        for (title, schema) in self.schemas.iter() {
            if let Some(jid) = &schema.jid {
                overall.insert(title, jid);
            }

            let mut class = json!({
                "@id": schema.jid,
                "@type": "rdfs:Class",
                "rdfs:label": schema.title,
                "rdfs:comment": schema.description,
            });

            if let Some(extends) = schema.extends.first() {
                if let Some(parent) = self.schemas.get(extends) {
                    class["rdfs:subClassOf"] = json!({ "@id": parent.jid });
                }
            }

            let mut graph = vec![class];

            for (property_name, property) in &schema.properties {
                if property_name == "type" || property_name == "id" {
                    continue;
                }

                if let Some(jid) = &property.jid {
                    overall.insert(property_name, jid);
                }

                let mut prop = json!({
                    "@id": property.jid,
                    "@type": "rdfs:Property",
                    "rdfs:label": property_name,
                    "rdfs:comment": property.description,
                });

                let domains = &domains[property_name];
                prop["schema:domainIncludes"] = if domains.len() == 1 {
                    json!({ "@id": domains[0] })
                } else {
                    // Sort lexically to avoid reordering on each generation
                    let sorted = domains
                        .iter()
                        .sorted()
                        .map(|id| json!({ "@id": id }))
                        .collect_vec();
                    json!(sorted)
                };

                let ranges = ranges(property, &self.schemas)
                    .iter()
                    .map(|id| json!({ "@id": id }))
                    .collect_vec();
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
                "@id": format!("https://stencila.org/{title}"),
                "name": title,
                //"version": version,
                "license": "https://creativecommons.org/publicdomain/zero/1.0/",
                "@context": {
                    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                    "schema": "https://schema.org/",
                    "stencila": "https://stencila.org/",
                },
                "@graph": graph
            });
            write(
                dir.join(format!("{title}.jsonld")),
                serde_json::to_string_pretty(&jsonld)?,
            )
            .await?
        }

        // Generate a `@context` with all the types and all the properties
        let jsonld = json!({
            "@id": "https://stencila.org/context",
            "name": "Stencila JSON-LD Context",
            //"version": version,
            "license": "https://creativecommons.org/publicdomain/zero/1.0/",
            "isBasedOn": ["https://schema.org/"],
            "@context": {
                "schema": "https://schema.org/",
                "stencila": "https://stencila.org/",
            },
            "@graph": overall
        });
        write(
            dir.join("context.jsonld"),
            serde_json::to_string_pretty(&jsonld)?,
        )
        .await?;

        Ok(())
    }
}

fn ranges(property: &Schema, schemas: &IndexMap<String, Schema>) -> Vec<String> {
    let mut ids = vec![];

    if let Some(r#type) = &property.r#type {
        match r#type {
            Type::Boolean => ids.push("schema:Boolean".to_string()),
            Type::Integer | Type::Number => ids.push("schema:Number".to_string()),
            Type::String => ids.push("schema:Text".to_string()),
            Type::Array => {
                if let Some(items) = &property.items {
                    match items {
                        Items::Ref(inner) => {
                            if let Some(jid) = schemas
                                .get(&inner.r#ref)
                                .and_then(|schema| schema.jid.as_ref())
                            {
                                ids.push(jid.clone())
                            }
                        }
                        Items::Type(inner) => ids.append(&mut ranges(
                            &Schema {
                                r#type: Some(inner.r#type.clone()),
                                ..Default::default()
                            },
                            schemas,
                        )),
                        Items::AnyOf(inner) => ids.append(&mut ranges(
                            &Schema {
                                any_of: Some(inner.any_of.clone()),
                                ..Default::default()
                            },
                            schemas,
                        )),
                        Items::List(inner) => ids.append(&mut ranges(
                            &Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            },
                            schemas,
                        )),
                    }
                }
            }
            _ => {}
        }
    } else if let Some(r#ref) = &property.r#ref {
        if let Some(jid) = schemas.get(r#ref).and_then(|schema| schema.jid.as_ref()) {
            ids.push(jid.clone())
        }
    } else if let Some(any_of) = &property.any_of {
        ids.append(
            &mut any_of
                .iter()
                .flat_map(|schema| ranges(schema, schemas))
                .collect_vec(),
        )
    }

    ids
}
