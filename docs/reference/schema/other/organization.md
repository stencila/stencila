---
title:
- type: Text
  value: Organization
---

# Organization

**An organization such as a school, NGO, corporation, club, etc.**

This is an implementation of schema.org [`Organization`](https://schema.org/Organization).


**`@id`**: [`schema:Organization`](https://schema.org/Organization)

## Properties

The `Organization` type has these properties:

| Name               | `@id`                                                                | Type                                                                                                                                                       | Description                                                                                                    | Inherited from                                                                  |
| ------------------ | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| id                 | [`schema:id`](https://schema.org/id)                                 | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                                                                   | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)             |
| alternateNames     | [`schema:alternateName`](https://schema.org/alternateName)           | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                                                                        | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| description        | [`schema:description`](https://schema.org/description)               | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                                                                     | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| identifiers        | [`schema:identifier`](https://schema.org/identifier)                 | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                                                                  | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| images             | [`schema:image`](https://schema.org/image)                           | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                                                                            | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| name               | [`schema:name`](https://schema.org/name)                             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                                                                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| url                | [`schema:url`](https://schema.org/url)                               | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                                                                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)               |
| address            | [`schema:address`](https://schema.org/address)                       | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) \| [`String`](https://stencila.dev/docs/reference/schema/data/string)   | Postal address for the organization.                                                                           | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| brands             | [`schema:brand`](https://schema.org/brand)                           | [`Brand`](https://stencila.dev/docs/reference/schema/other/brand)*                                                                                         | Brands that the organization is connected with.                                                                | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| contactPoints      | [`schema:contactPoint`](https://schema.org/contactPoint)             | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point)*                                                                          | Correspondence/Contact points for the organization.                                                            | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| departments        | [`schema:department`](https://schema.org/department)                 | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization)*                                                                           | Departments within the organization. For example, Department of Computer Science, Research & Development etc.  | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| funders            | [`schema:funder`](https://schema.org/funder)                         | ([`Organization`](https://stencila.dev/docs/reference/schema/other/organization) \| [`Person`](https://stencila.dev/docs/reference/schema/other/person))*  | Organization(s) or person(s) funding the organization.                                                         | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| legalName          | [`schema:legalName`](https://schema.org/legalName)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The official name of the organization, e.g. the registered company name.                                       | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| logo               | [`schema:logo`](https://schema.org/logo)                             | [`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The logo of the organization.                                                                                  | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| members            | [`schema:member`](https://schema.org/member)                         | ([`Organization`](https://stencila.dev/docs/reference/schema/other/organization) \| [`Person`](https://stencila.dev/docs/reference/schema/other/person))*  | Person(s) or organization(s) who are members of this organization.                                             | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |
| parentOrganization | [`schema:parentOrganization`](https://schema.org/parentOrganization) | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization)                                                                            | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.    | [`Organization`](https://stencila.dev/docs/reference/schema/other/organization) |

## Related

The `Organization` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `Organization` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                                           |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | --------------------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                                                 |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<institution>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/institution) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |                                                                                                                 |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                                 |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                                 |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                                 |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                                 |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                                 |

## Bindings

The `Organization` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Organization.jsonld)
- [JSON Schema](https://stencila.dev/Organization.schema.json)
- Python class [`Organization`](https://github.com/stencila/stencila/blob/main/python/stencila/types/organization.py)
- Rust struct [`Organization`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/organization.rs)
- TypeScript class [`Organization`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Organization.ts)

## Source

This documentation was generated from [`Organization.yaml`](https://github.com/stencila/stencila/blob/main/schema/Organization.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).