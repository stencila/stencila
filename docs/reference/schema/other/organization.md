---
title: Organization
description: An organization such as a school, NGO, corporation, club, etc.
config:
  publish:
    ghost:
      type: post
      slug: organization
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

This is an implementation of schema.org [`Organization`](https://schema.org/Organization).


# Properties

The `Organization` type has these properties:

| Name                 | Description                                                                                                   | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                                        | Aliases                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`                 | The identifier for this item.                                                                                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                                 | -                                                                                         |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName)           | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`        | A description of the item.                                                                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)               | -                                                                                         |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                 | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)                 | `identifier`                                                                              |
| `images`             | Images of the item.                                                                                           | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                           | `image`                                                                                   |
| `name`               | The name of the item.                                                                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                             | -                                                                                         |
| `url`                | The URL of the item.                                                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                               | -                                                                                         |
| `ror`                | The organization's Research Organization Registry ID (https://ror.org/).                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | `stencila:ror`                                                       | -                                                                                         |
| `address`            | Postal address for the organization.                                                                          | [`PostalAddress`](https://stencila.ghost.io/docs/reference/schema/postal-address) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)    | -                                                                  | [`schema:address`](https://schema.org/address)                       | -                                                                                         |
| `brands`             | Brands that the organization is connected with.                                                               | [`Brand`](https://stencila.ghost.io/docs/reference/schema/brand)*                                                                                          | -                                                                  | [`schema:brand`](https://schema.org/brand)                           | `brand`                                                                                   |
| `contactPoints`      | Correspondence/Contact points for the organization.                                                           | [`ContactPoint`](https://stencila.ghost.io/docs/reference/schema/contact-point)*                                                                           | -                                                                  | [`schema:contactPoint`](https://schema.org/contactPoint)             | `contact-points`, `contact_points`, `contactPoint`, `contact-point`, `contact_point`      |
| `departments`        | Departments within the organization. For example, Department of Computer Science, Research & Development etc. | [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization)*                                                                            | -                                                                  | [`schema:department`](https://schema.org/department)                 | `department`                                                                              |
| `funders`            | Organization(s) or person(s) funding the organization.                                                        | ([`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization))*    | -                                                                  | [`schema:funder`](https://schema.org/funder)                         | `funder`                                                                                  |
| `legalName`          | The official name of the organization, e.g. the registered company name.                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:legalName`](https://schema.org/legalName)                   | `legal-name`, `legal_name`                                                                |
| `logo`               | The logo of the organization.                                                                                 | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)                                                                              | -                                                                  | [`schema:logo`](https://schema.org/logo)                             | -                                                                                         |
| `members`            | Person(s) or organization(s) who are members of this organization.                                            | ([`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization))*    | -                                                                  | [`schema:member`](https://schema.org/member)                         | `member`                                                                                  |
| `parentOrganization` | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.   | [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization)                                                                             | -                                                                  | [`schema:parentOrganization`](https://schema.org/parentOrganization) | `parent-organization`, `parent_organization`                                              |

# Related

The `Organization` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: none

# Formats

The `Organization` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                          | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                                  |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                                                                                                  |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   |              | Encoded as [`<institution>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/institution.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              |                                                                                                                  |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                                                                                                  |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                                                                                                  |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                                                                                                  |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                                                                                                  |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                  |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                                  |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                                  |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                                  |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                                  |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                                  |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                                  |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                                  |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                                  |

# Bindings

The `Organization` type is represented in:

- [JSON-LD](https://stencila.org/Organization.jsonld)
- [JSON Schema](https://stencila.org/Organization.schema.json)
- Python class [`Organization`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/organization.py)
- Rust struct [`Organization`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/organization.rs)
- TypeScript class [`Organization`](https://github.com/stencila/stencila/blob/main/ts/src/types/Organization.ts)

# Source

This documentation was generated from [`Organization.yaml`](https://github.com/stencila/stencila/blob/main/schema/Organization.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
