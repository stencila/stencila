# Monetary Grant

**A monetary grant.**

This type is an implementation of [schema:MonetaryGrant](https://schema.org/MonetaryGrant).

## Properties

| Name           | `@id`                                                    | Type                                                                                                 | Description                                                                                                                    | Inherited from                    |
| -------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------- |
| alternateNames | [schema:alternateName](https://schema.org/alternateName) | Array of string                                                                                      | Alternate names (aliases) for the item.                                                                                        | [Thing](Thing.md)                 |
| amounts        | [schema:amount](https://schema.org/amount)               | number                                                                                               | The amount of money. See note [1](#notes).                                                                                     | [MonetaryGrant](MonetaryGrant.md) |
| description    | [schema:description](https://schema.org/description)     | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [2](#notes).                                                                               | [Thing](Thing.md)                 |
| fundedItems    | [schema:fundedItem](https://schema.org/fundedItem)       | Array of [Thing](Thing.md)                                                                           | Indicates an item funded or sponsored through a Grant. See note [3](#notes).                                                   | [Grant](Grant.md)                 |
| funders        | [schema:funder](https://schema.org/funder)               | Array of ([Person](Person.md) _or_ [Organization](Organization.md))                                  | A person or organization that supports (sponsors) something through some kind of financial contribution. See note [4](#notes). | [MonetaryGrant](MonetaryGrant.md) |
| id             | [schema:id](https://schema.org/id)                       | string                                                                                               | The identifier for this item.                                                                                                  | [Entity](Entity.md)               |
| identifiers    | [schema:identifier](https://schema.org/identifier)       | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [5](#notes).                                                            | [Thing](Thing.md)                 |
| images         | [schema:image](https://schema.org/image)                 | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                                                                            | [Thing](Thing.md)                 |
| meta           | [stencila:meta](https://schema.stenci.la/meta.jsonld)    | object                                                                                               | Metadata associated with this item.                                                                                            | [Entity](Entity.md)               |
| name           | [schema:name](https://schema.org/name)                   | string                                                                                               | The name of the item.                                                                                                          | [Thing](Thing.md)                 |
| sponsors       | [schema:sponsor](https://schema.org/sponsor)             | Array of ([Person](Person.md) _or_ [Organization](Organization.md))                                  | A person or organization that supports a thing through a pledge, promise, or financial contribution. See note [6](#notes).     | [Grant](Grant.md)                 |
| url            | [schema:url](https://schema.org/url)                     | Format 'uri'                                                                                         | The URL of the item.                                                                                                           | [Thing](Thing.md)                 |

## Notes

1. **amounts** : This property is an array, to allow for multiple amounts, possibly, but not necessarily, from multiple `funders`.
2. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
3. **fundedItems** : An array because a grant may have more than one funded item.
4. **funders** : Often a `MonetaryGrant` will have a single funder. This property is an array to be consistent with `funders` on other types e.g. `CreativeWork`.
5. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).
6. **sponsors** : An array because a grant may have more than one sponsor.

## Related

- Parent: [Grant](Grant.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/MonetaryGrant.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/MonetaryGrant.schema.json)
- Python [`class MonetaryGrant`](https://stencila.github.io/schema/python/docs/types.html#schema.types.MonetaryGrant)
- TypeScript [`interface MonetaryGrant`](https://stencila.github.io/schema/ts/docs/interfaces/monetarygrant.html)
- R [`class MonetaryGrant`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct MonetaryGrant`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.MonetaryGrant.html)

## Source

This documentation was generated from [MonetaryGrant.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/MonetaryGrant.schema.yaml).
