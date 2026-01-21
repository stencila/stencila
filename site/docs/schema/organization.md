---
title: Organization
description: An organization such as a school, NGO, corporation, club, etc.
---

This is an implementation of schema.org [`Organization`](https://schema.org/Organization).


# Properties

The `Organization` type has these properties:

| Name                 | Description                                                                                                   | Type                                                                 | Inherited from          |
| -------------------- | ------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`                 | The identifier for this item.                                                                                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`        | A description of the item.                                                                                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                 | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`             | Images of the item.                                                                                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`               | The name of the item.                                                                                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`                | The URL of the item.                                                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `ror`                | The organization's Research Organization Registry ID (https://ror.org/).                                      | [`String`](./string.md)                                              | -                       |
| `address`            | Postal address for the organization.                                                                          | [`PostalAddress`](./postal-address.md) \| [`String`](./string.md)    | -                       |
| `brands`             | Brands that the organization is connected with.                                                               | [`Brand`](./brand.md)*                                               | -                       |
| `contactPoints`      | Correspondence/Contact points for the organization.                                                           | [`ContactPoint`](./contact-point.md)*                                | -                       |
| `departments`        | Departments within the organization. For example, Department of Computer Science, Research & Development etc. | [`Organization`](./organization.md)*                                 | -                       |
| `funders`            | Organization(s) or person(s) funding the organization.                                                        | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       |
| `legalName`          | The official name of the organization, e.g. the registered company name.                                      | [`String`](./string.md)                                              | -                       |
| `logo`               | The logo of the organization.                                                                                 | [`ImageObject`](./image-object.md)                                   | -                       |
| `members`            | Person(s) or organization(s) who are members of this organization.                                            | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       |
| `parentOrganization` | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.   | [`Organization`](./organization.md)                                  | -                       |

# Related

The `Organization` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Organization` type is represented in:

- [JSON-LD](https://stencila.org/Organization.jsonld)
- [JSON Schema](https://stencila.org/Organization.schema.json)
- Python class [`Organization`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/organization.py)
- Rust struct [`Organization`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/organization.rs)
- TypeScript class [`Organization`](https://github.com/stencila/stencila/blob/main/ts/src/types/Organization.ts)

# Source

This documentation was generated from [`Organization.yaml`](https://github.com/stencila/stencila/blob/main/schema/Organization.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
