# stencila-schema-json

Provides simplified, self-contained JSON Schema representations of Stencila schema types for use with external services such as LLM APIs.

## Purpose

This crate is needed because the existing `../../json/*.schema.json` files contain Stencila-specific extensions and external references that are incompatible with standard JSON Schema validators and external services like LLM APIs.

The generated schemas are standard-compliant, self-contained (using internal `#/definitions/...` references), and free of custom properties like `"extends": [...]` and format-specific metadata that external services cannot process.

## Provider Compatibility

While this crate generates standard JSON Schema, different LLM providers support varying subsets of the specification.

### OpenAI

OpenAI's Structured Outputs feature supports most JSON Schema features but has some specific requirements and limitations including `additionalProperties: false` on all objects.

See: https://platform.openai.com/docs/guides/structured-outputs#supported-schemas

### Google Gemini

Google Gemini's `responseJsonSchema` supports a limited subset of JSON Schema. At the time of writing,supported properties include:

- `$id`, `$defs`, `$ref`, `$anchor`
- `type`, `format`, `title`, `description`
- `enum` (for strings and numbers)
- `items`, `prefixItems`, `minItems`, `maxItems`
- `minimum`, `maximum`
- `anyOf`, `oneOf` (interpreted as `anyOf`)
- `properties`, `additionalProperties`, `required`

Notably **not supported**:

- `const` (use single-value `enum` instead)
- `allOf`, `not`
- Complex string validation (`pattern`, `minLength`, `maxLength`)

See: https://ai.google.dev/api/generate-content#FIELDS.response_json_schema

### Mistral

Mistral requires explicit `additionalProperties: false` on objects to prevent extra properties.

See: https://docs.mistral.ai/capabilities/function_calling/
