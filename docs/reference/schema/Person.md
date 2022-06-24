# Person

**A person (alive, dead, undead, or fictional).**

This type is an implementation of [schema:Person](https://schema.org/Person).

## Properties

| Name             | `@id`                                                        | Type                                                                                                 | Description                                                                                              | Inherited from      |
| ---------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------- |
| address          | [schema:address](https://schema.org/address)                 | [PostalAddress](PostalAddress.md) _or_ string                                                        | Postal address for the person.                                                                           | [Person](Person.md) |
| affiliations     | [schema:affiliation](https://schema.org/affiliation)         | Array of [Organization](Organization.md)                                                             | Organizations that the person is affiliated with.                                                        | [Person](Person.md) |
| alternateNames   | [schema:alternateName](https://schema.org/alternateName)     | Array of string                                                                                      | Alternate names (aliases) for the item.                                                                  | [Thing](Thing.md)   |
| description      | [schema:description](https://schema.org/description)         | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [1](#notes).                                                         | [Thing](Thing.md)   |
| emails           | [schema:email](https://schema.org/email)                     | Array of Format 'email'                                                                              | Email addresses for the person.                                                                          | [Person](Person.md) |
| familyNames      | [schema:familyName](https://schema.org/familyName)           | Parser 'ssi' _and_ Array of string                                                                   | Family name. In the U.S., the last name of a person. See note [2](#notes).                               | [Person](Person.md) |
| funders          | [schema:funder](https://schema.org/funder)                   | Array of ([Organization](Organization.md) _or_ [Person](Person.md))                                  | A person or organization that supports (sponsors) something through some kind of financial contribution. | [Person](Person.md) |
| givenNames       | [schema:givenName](https://schema.org/givenName)             | Parser 'ssi' _and_ Array of string                                                                   | Given name. In the U.S., the first name of a person. See note [3](#notes).                               | [Person](Person.md) |
| honorificPrefix  | [schema:honorificPrefix](https://schema.org/honorificPrefix) | string                                                                                               | An honorific prefix preceding a person's name such as Dr/Mrs/Mr.                                         | [Person](Person.md) |
| honorificSuffix  | [schema:honorificSuffix](https://schema.org/honorificSuffix) | string                                                                                               | An honorific suffix after a person's name such as MD/PhD/MSCSW.                                          | [Person](Person.md) |
| id               | [schema:id](https://schema.org/id)                           | string                                                                                               | The identifier for this item.                                                                            | [Entity](Entity.md) |
| identifiers      | [schema:identifier](https://schema.org/identifier)           | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [4](#notes).                                      | [Thing](Thing.md)   |
| images           | [schema:image](https://schema.org/image)                     | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                                                      | [Thing](Thing.md)   |
| jobTitle         | [schema:jobTitle](https://schema.org/jobTitle)               | string                                                                                               | The job title of the person (for example, Financial Manager).                                            | [Person](Person.md) |
| memberOf         | [schema:memberOf](https://schema.org/memberOf)               | Array of [Organization](Organization.md)                                                             | An organization (or program membership) to which this person belongs.                                    | [Person](Person.md) |
| meta             | [stencila:meta](https://schema.stenci.la/meta.jsonld)        | object                                                                                               | Metadata associated with this item.                                                                      | [Entity](Entity.md) |
| name             | [schema:name](https://schema.org/name)                       | string                                                                                               | The name of the item.                                                                                    | [Thing](Thing.md)   |
| telephoneNumbers | [schema:telephone](https://schema.org/telephone)             | Array of string                                                                                      | Telephone numbers for the person.                                                                        | [Person](Person.md) |
| url              | [schema:url](https://schema.org/url)                         | Format 'uri'                                                                                         | The URL of the item.                                                                                     | [Thing](Thing.md)   |

## Notes

1. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
2. **familyNames** : This can be used along with givenName instead of the name property.
3. **givenNames** : This can be used along with familyName instead of the name property.
4. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).

## Examples

```json
{
  "type": "Person",
  "honorificPrefix": "Dr",
  "givenNames": ["Marie", "Skłodowska"],
  "familyNames": ["Curie"],
  "honorificSuffix": "PhD"
}
```

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Person.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Person.schema.json)
- Python [`class Person`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Person)
- TypeScript [`interface Person`](https://stencila.github.io/schema/ts/docs/interfaces/person.html)
- R [`class Person`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Person`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Person.html)

## Source

This documentation was generated from [Person.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Person.schema.yaml).
