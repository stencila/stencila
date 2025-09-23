use strum::EnumString;

mod schema;

mod blocks;
mod inlines;
mod other;
mod works;

pub use schema::JsonSchema;

/// JSON Schema variants available for generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum JsonSchemaVariant {
    /// Article metadata
    ArticleMetadata,

    /// Simple article containing simple block content
    ArticleSimple,

    /// Reference of any type
    ReferenceAny,

    /// Simple representation of a person including identifiers and affiliations
    PersonSimple,

    /// Simple representation of an organization including identifiers and address
    OrganizationSimple,

    /// Simple block content
    BlockSimple,

    /// Simple inline content
    InlineSimple,

    /// Simple paragraph
    ParagraphSimple,

    /// Simple table without captions or notes
    TableSimple,

    /// Block math in TeX format
    MathBlockTex,

    /// Inline math in TeX format
    MathInlineTex,

    /// Plain text content
    TextSimple,
}

/// Get a JSON Schema by variant
pub fn json_schema(variant: JsonSchemaVariant) -> JsonSchema {
    use JsonSchemaVariant::*;
    match variant {
        ArticleMetadata => JsonSchema::standalone(works::article::metadata()),
        ArticleSimple => JsonSchema::standalone(works::article::simple()),

        ReferenceAny => JsonSchema::standalone(works::reference::reference()),

        PersonSimple => JsonSchema::standalone(other::person::simple()),
        OrganizationSimple => JsonSchema::standalone(other::organization::simple()),

        InlineSimple => JsonSchema::standalone(inlines::simple()),
        TextSimple => JsonSchema::standalone(inlines::text::plain()),
        MathInlineTex => JsonSchema::standalone(inlines::math::tex()),

        BlockSimple => JsonSchema::standalone(blocks::simple()),
        ParagraphSimple => JsonSchema::standalone(blocks::paragraph::simple()),
        TableSimple => JsonSchema::standalone(blocks::table::simple()),
        MathBlockTex => JsonSchema::standalone(blocks::math::tex()),
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_json_schema_variant_parsing() -> Result<()> {
        assert_eq!(
            JsonSchemaVariant::from_str("article-metadata")?,
            JsonSchemaVariant::ArticleMetadata
        );
        assert_eq!(
            JsonSchemaVariant::from_str("article-simple")?,
            JsonSchemaVariant::ArticleSimple
        );
        assert_eq!(
            JsonSchemaVariant::from_str("math-inline-tex")?,
            JsonSchemaVariant::MathInlineTex
        );
        assert_eq!(
            JsonSchemaVariant::from_str("math-block-tex")?,
            JsonSchemaVariant::MathBlockTex
        );

        Ok(())
    }

    #[test]
    fn test_json_schema_generation() -> Result<()> {
        let schema = json_schema(JsonSchemaVariant::ArticleSimple);
        // Just check that we can generate a schema without errors
        assert!(serde_json::to_string(&schema).is_ok());

        Ok(())
    }
}
