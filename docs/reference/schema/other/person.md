---
title:
- type: Text
  value: Person
---

# Person

**A person (alive, dead, undead, or fictional).**

**`@id`**: [`schema:Person`](https://schema.org/Person)

## Properties

The `Person` type has these properties:

| Name             | `@id`                                                          | Type                                                                                                                                                       | Description                                                                                               | Inherited from                                                      |
| ---------------- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| id               | [`schema:id`](https://schema.org/id)                           | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                                                              | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| alternateNames   | [`schema:alternateName`](https://schema.org/alternateName)     | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                                                                   | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| description      | [`schema:description`](https://schema.org/description)         | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                                                                | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| identifiers      | [`schema:identifier`](https://schema.org/identifier)           | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                                                             | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| images           | [`schema:image`](https://schema.org/image)                     | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                                                                       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| name             | [`schema:name`](https://schema.org/name)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                                                                     | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| url              | [`schema:url`](https://schema.org/url)                         | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                                                                      | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| address          | [`schema:address`](https://schema.org/address)                 | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) \| [`String`](https://stencila.dev/docs/reference/schema/data/string)   | Postal address for the person.                                                                            | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| affiliations     | [`schema:affiliation`](https://schema.org/affiliation)         | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization)*                                                                           | Organizations that the person is affiliated with.                                                         | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| emails           | [`schema:email`](https://schema.org/email)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Email addresses for the person.                                                                           | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| familyNames      | [`schema:familyName`](https://schema.org/familyName)           | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Family name. In the U.S., the last name of a person.                                                      | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| funders          | [`schema:funder`](https://schema.org/funder)                   | ([`Organization`](https://stencila.dev/docs/reference/schema/other/organization) \| [`Person`](https://stencila.dev/docs/reference/schema/other/person))*  | A person or organization that supports (sponsors) something through some kind of financial contribution.  | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| givenNames       | [`schema:givenName`](https://schema.org/givenName)             | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Given name. In the U.S., the first name of a person.                                                      | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| honorificPrefix  | [`schema:honorificPrefix`](https://schema.org/honorificPrefix) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | An honorific prefix preceding a person's name such as Dr/Mrs/Mr.                                          | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| honorificSuffix  | [`schema:honorificSuffix`](https://schema.org/honorificSuffix) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | An honorific suffix after a person's name such as MD/PhD/MSCSW.                                           | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| jobTitle         | [`schema:jobTitle`](https://schema.org/jobTitle)               | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The job title of the person (for example, Financial Manager).                                             | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| memberOf         | [`schema:memberOf`](https://schema.org/memberOf)               | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization)*                                                                           | An organization (or program membership) to which this person belongs.                                     | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |
| telephoneNumbers | [`schema:telephone`](https://schema.org/telephone)             | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Telephone numbers for the person.                                                                         | [`Person`](https://stencila.dev/docs/reference/schema/other/person) |

## Related

The `Person` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `Person` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Person` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Person.jsonld)
- [JSON Schema](https://stencila.dev/Person.schema.json)
- Python class [`Person`](https://github.com/stencila/stencila/blob/main/python/stencila/types/person.py)
- Rust struct [`Person`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/person.rs)
- TypeScript class [`Person`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Person.ts)

## Source

This documentation was generated from [`Person.yaml`](https://github.com/stencila/stencila/blob/main/schema/Person.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).