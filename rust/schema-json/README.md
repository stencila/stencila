# stencila-schema-json

Provides simplified, self-contained JSON Schema representations of Stencila schema types for use with external services such as LLM APIs.

## Purpose

This crate is needed because the existing `../../json/*.schema.json` files contain Stencila-specific extensions and external references that are incompatible with standard JSON Schema validators and external services like LLM APIs.

The generated schemas are standard-compliant, self-contained (using internal `#/definitions/...` references), and free of custom properties like `"extends": [...]` and format-specific metadata that external services cannot process.

## Provider Compatibility

While this crate generates standard JSON Schema, different LLM providers support varying subsets of the specification. This crate provides transformation functions to automatically make schemas compatible with specific providers.

### Usage

Apply provider-specific transformations using these methods:

```rust
// Make schema compatible with specific providers
let schema = json_schema("article:metadata")?;
let mistral_schema = schema.clone().for_mistral();
let google_schema = schema.clone().for_google();
let openai_schema = schema.clone().for_openai();
```

### OpenAI

Use `.for_openai()` to make schemas compatible with OpenAI's Structured Outputs feature. This removes unsupported features like `allOf` and `default` values.

See: https://platform.openai.com/docs/guides/structured-outputs#supported-schemas

### Google Gemini

Use `.for_google()` to make schemas compatible with Google Gemini's `responseJsonSchema`. This converts `const` to single-value `enum` and removes unsupported features like `pattern`.

See: https://ai.google.dev/api/generate-content#FIELDS.response_json_schema

### Mistral

Use `.for_mistral()` to make schemas compatible with Mistral's function calling. This removes unsupported features like `format` and `pattern` validation.

See: https://docs.mistral.ai/capabilities/function_calling/
