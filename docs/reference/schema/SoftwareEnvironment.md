# Software Environment

**A computational environment.**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Properties

| Name           | `@id`                                                       | Type                                                                                                 | Description                                                                                             | Inherited from                                |
| -------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| **name**       | [schema:name](https://schema.org/name)                      | string                                                                                               | The name of the item.                                                                                   | [Thing](Thing.md)                             |
| adds           | [stencila:adds](https://schema.stenci.la/adds.jsonld)       | Array of [SoftwareSourceCode](SoftwareSourceCode.md)                                                 | The packages that this environment adds to the base environments listed under `extends` (if any).,      | [SoftwareEnvironment](SoftwareEnvironment.md) |
| alternateNames | [schema:alternateName](https://schema.org/alternateName)    | Array of string                                                                                      | Alternate names (aliases) for the item.                                                                 | [Thing](Thing.md)                             |
| description    | [schema:description](https://schema.org/description)        | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [1](#notes).                                                        | [Thing](Thing.md)                             |
| extends        | [stencila:extends](https://schema.stenci.la/extends.jsonld) | Array of [SoftwareEnvironment](SoftwareEnvironment.md)                                               | Other environments that this environment extends by adding or removing packages.,                       | [SoftwareEnvironment](SoftwareEnvironment.md) |
| id             | [schema:id](https://schema.org/id)                          | string                                                                                               | The identifier for this item.                                                                           | [Entity](Entity.md)                           |
| identifiers    | [schema:identifier](https://schema.org/identifier)          | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [2](#notes).                                     | [Thing](Thing.md)                             |
| images         | [schema:image](https://schema.org/image)                    | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                                                     | [Thing](Thing.md)                             |
| meta           | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                                                                               | Metadata associated with this item.                                                                     | [Entity](Entity.md)                           |
| removes        | [stencila:removes](https://schema.stenci.la/removes.jsonld) | Array of [SoftwareSourceCode](SoftwareSourceCode.md)                                                 | The packages that this environment removes from the base environments listed under `extends` (if any)., | [SoftwareEnvironment](SoftwareEnvironment.md) |
| url            | [schema:url](https://schema.org/url)                        | Format 'uri'                                                                                         | The URL of the item.                                                                                    | [Thing](Thing.md)                             |

## Notes

1. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
2. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/SoftwareEnvironment.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/SoftwareEnvironment.schema.json)
- Python [`class SoftwareEnvironment`](https://stencila.github.io/schema/python/docs/types.html#schema.types.SoftwareEnvironment)
- TypeScript [`interface SoftwareEnvironment`](https://stencila.github.io/schema/ts/docs/interfaces/softwareenvironment.html)
- R [`class SoftwareEnvironment`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct SoftwareEnvironment`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.SoftwareEnvironment.html)

## Source

This documentation was generated from [SoftwareEnvironment.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/SoftwareEnvironment.schema.yaml).
