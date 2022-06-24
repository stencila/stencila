# Date

**A date encoded as a ISO 8601 string.**

This type serves mainly to disambiguate an ISO 8601 date string from a plain string. It should generally be used instead of a date formatted string.

## Properties

| Name      | `@id`                                                 | Type                                              | Description                         | Inherited from      |
| --------- | ----------------------------------------------------- | ------------------------------------------------- | ----------------------------------- | ------------------- |
| **value** | [schema:value](https://schema.org/value)              | Format 'date-time' _or_ Format 'date' _or_ string | The date as an ISO 8601 string.     | [Date](Date.md)     |
| id        | [schema:id](https://schema.org/id)                    | string                                            | The identifier for this item.       | [Entity](Entity.md) |
| meta      | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object                                            | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Date.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Date.schema.json)
- Python [`class Date`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Date)
- TypeScript [`interface Date`](https://stencila.github.io/schema/ts/docs/interfaces/date.html)
- R [`class Date`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Date`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Date.html)

## Source

This documentation was generated from [Date.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Date.schema.yaml).
