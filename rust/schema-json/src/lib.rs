use eyre::{Result, bail};

mod schema;

mod blocks;
mod inlines;
mod other;
mod works;

pub use schema::JsonSchema;

/// Get a JSON Schema by name
/// 
/// The naming convention for schemas if the kebab-cased name of the Stencila Schema type
/// (e.g. `article`, `math-inline`) followed by a colon and the name of the variant (e.g. `simple`).
pub fn json_schema(name: &str) -> Result<JsonSchema> {
    match name {
        "article:simple" => Ok(JsonSchema::standalone(works::article::simple())),
        "article:metadata" => Ok(JsonSchema::standalone(works::article::metadata())),
        "periodical:simple" => Ok(JsonSchema::standalone(works::periodical::simple())),
        
        "person:simple" => Ok(JsonSchema::standalone(other::person::simple())),
        "organization:simple" => Ok(JsonSchema::standalone(other::organization::simple())),
        
        "inline:simple" => Ok(JsonSchema::standalone(inlines::simple())),
        "text:simple" => Ok(JsonSchema::standalone(inlines::text::plain())),
        "math-inline:tex" => Ok(JsonSchema::standalone(inlines::math::tex())),
        
        "block:simple" => Ok(JsonSchema::standalone(blocks::simple())),
        "paragraph:simple" => Ok(JsonSchema::standalone(blocks::paragraph::simple())),
        "math-block:tex" => Ok(JsonSchema::standalone(blocks::math::tex())),
        
        _ => bail!("Unknown schema: {name}"),
    }
}
