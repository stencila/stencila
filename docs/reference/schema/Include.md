# Include

**Include content from an external source (e.g. file, URL).**

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name        | `@id`                                                               | Type                                     | Description                                                                                                           | Inherited from        |
| ----------- | ------------------------------------------------------------------- | ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | --------------------- |
| **source**  | [stencila:source](https://schema.stenci.la/source.jsonld)           | string                                   | The external source of the content, a file path or URL.                                                               | [Include](Include.md) |
| buildDigest | [stencila:buildDigest](https://schema.stenci.la/buildDigest.jsonld) | string                                   | The SHA-256 digest of the `source` and `mediaType` properties the last time the node was built. See note [1](#notes). | [Include](Include.md) |
| content     | [stencila:content](https://schema.stenci.la/content.jsonld)         | Array of [BlockContent](BlockContent.md) | The structured content decoded from the source. See note [2](#notes).                                                 | [Include](Include.md) |
| id          | [schema:id](https://schema.org/id)                                  | string                                   | The identifier for this item.                                                                                         | [Entity](Entity.md)   |
| mediaType   | [schema:encodingFormat](https://schema.org/encodingFormat)          | string                                   | Media type of the source content. See note [3](#notes).                                                               | [Include](Include.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)               | object                                   | Metadata associated with this item.                                                                                   | [Entity](Entity.md)   |

## Notes

1. **buildDigest** : Used to determine whether it is necessary to re-build the node (i.e. update the `content` property due to new content in the `source` or a change in the `mediaType`).
2. **content** : Assumes that included content will be block content i.e. that there will be limited instances where a user would want to use an `Include` node to transclude inline content.
3. **mediaType** : Typically expressed using a file name extensions (e.g. `md`) or a MIME type (e.g. `text/md`).

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Include.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Include.schema.json)
- Python [`class Include`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Include)
- TypeScript [`interface Include`](https://stencila.github.io/schema/ts/docs/interfaces/include.html)
- R [`class Include`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Include`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Include.html)

## Source

This documentation was generated from [Include.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Include.schema.yaml).
