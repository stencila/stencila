---
title: Person
description: A person (alive, dead, undead, or fictional).
---

# Properties

The `Person` type has these properties:

| Name               | Description                                                                                              | Type                                                                 | Inherited from          |
| ------------------ | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`               | The identifier for this item.                                                                            | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames`   | Alternate names (aliases) for the item.                                                                  | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`      | A description of the item.                                                                               | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`      | Any kind of identifier for any kind of Thing.                                                            | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`           | Images of the item.                                                                                      | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`             | The name of the item.                                                                                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`              | The URL of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `orcid`            | The person's Open Researcher and Contributor ID (https://orcid.org/).                                    | [`String`](./string.md)                                              | -                       |
| `address`          | Postal address for the person.                                                                           | [`PostalAddress`](./postal-address.md) \| [`String`](./string.md)    | -                       |
| `affiliations`     | Organizations that the person is affiliated with.                                                        | [`Organization`](./organization.md)*                                 | -                       |
| `emails`           | Email addresses for the person.                                                                          | [`String`](./string.md)*                                             | -                       |
| `familyNames`      | Family name. In the U.S., the last name of a person.                                                     | [`String`](./string.md)*                                             | -                       |
| `funders`          | A person or organization that supports (sponsors) something through some kind of financial contribution. | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       |
| `givenNames`       | Given name. In the U.S., the first name of a person.                                                     | [`String`](./string.md)*                                             | -                       |
| `honorificPrefix`  | An honorific prefix preceding a person's name such as Dr/Mrs/Mr.                                         | [`String`](./string.md)                                              | -                       |
| `honorificSuffix`  | An honorific suffix after a person's name such as MD/PhD/MSCSW.                                          | [`String`](./string.md)                                              | -                       |
| `jobTitle`         | The job title of the person (for example, Financial Manager).                                            | [`String`](./string.md)                                              | -                       |
| `memberOf`         | An organization (or program membership) to which this person belongs.                                    | [`Organization`](./organization.md)*                                 | -                       |
| `telephoneNumbers` | Telephone numbers for the person.                                                                        | [`String`](./string.md)*                                             | -                       |

# Related

The `Person` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Person` type is represented in:

- [JSON-LD](https://stencila.org/Person.jsonld)
- [JSON Schema](https://stencila.org/Person.schema.json)
- Python class [`Person`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/person.py)
- Rust struct [`Person`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/person.rs)
- TypeScript class [`Person`](https://github.com/stencila/stencila/blob/main/ts/src/types/Person.ts)

# Source

This documentation was generated from [`Person.yaml`](https://github.com/stencila/stencila/blob/main/schema/Person.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
