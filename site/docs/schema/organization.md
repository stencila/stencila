---
title: Organization
description: An organization such as a school, NGO, corporation, club, etc.
---

This is an implementation of schema.org [`Organization`](https://schema.org/Organization).


# Properties

The `Organization` type has these properties:

| Name                 | Description                                                                                                   | Type                                                                 | Inherited from          | `JSON-LD @id`                                                        | Aliases                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`                 | The identifier for this item.                                                                                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                                 | -                                                                                         |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName)           | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`        | A description of the item.                                                                                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)               | -                                                                                         |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                 | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)                 | `identifier`                                                                              |
| `images`             | Images of the item.                                                                                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                           | `image`                                                                                   |
| `name`               | The name of the item.                                                                                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                             | -                                                                                         |
| `url`                | The URL of the item.                                                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                               | -                                                                                         |
| `ror`                | The organization's Research Organization Registry ID (https://ror.org/).                                      | [`String`](./string.md)                                              | -                       | `stencila:ror`                                                       | -                                                                                         |
| `address`            | Postal address for the organization.                                                                          | [`PostalAddress`](./postal-address.md) \| [`String`](./string.md)    | -                       | [`schema:address`](https://schema.org/address)                       | -                                                                                         |
| `brands`             | Brands that the organization is connected with.                                                               | [`Brand`](./brand.md)*                                               | -                       | [`schema:brand`](https://schema.org/brand)                           | `brand`                                                                                   |
| `contactPoints`      | Correspondence/Contact points for the organization.                                                           | [`ContactPoint`](./contact-point.md)*                                | -                       | [`schema:contactPoint`](https://schema.org/contactPoint)             | `contact-points`, `contact_points`, `contactPoint`, `contact-point`, `contact_point`      |
| `departments`        | Departments within the organization. For example, Department of Computer Science, Research & Development etc. | [`Organization`](./organization.md)*                                 | -                       | [`schema:department`](https://schema.org/department)                 | `department`                                                                              |
| `funders`            | Organization(s) or person(s) funding the organization.                                                        | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       | [`schema:funder`](https://schema.org/funder)                         | `funder`                                                                                  |
| `legalName`          | The official name of the organization, e.g. the registered company name.                                      | [`String`](./string.md)                                              | -                       | [`schema:legalName`](https://schema.org/legalName)                   | `legal-name`, `legal_name`                                                                |
| `logo`               | The logo of the organization.                                                                                 | [`ImageObject`](./image-object.md)                                   | -                       | [`schema:logo`](https://schema.org/logo)                             | -                                                                                         |
| `members`            | Person(s) or organization(s) who are members of this organization.                                            | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       | [`schema:member`](https://schema.org/member)                         | `member`                                                                                  |
| `parentOrganization` | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.   | [`Organization`](./organization.md)                                  | -                       | [`schema:parentOrganization`](https://schema.org/parentOrganization) | `parent-organization`, `parent_organization`                                              |

# Related

The `Organization` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Formats

The `Organization` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                          | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                  |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                                  |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   |              | Encoded as [`<institution>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/institution.html) |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |                                                                                                                  |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                                                                                                  |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                                                                                                  |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                                                                                                  |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                                                                                                  |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                  |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                  |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                  |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                  |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                  |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                  |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                  |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                  |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                  |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                  |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                  |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                  |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                  |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                  |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                  |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                  |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                  |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                  |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                  |

# Bindings

The `Organization` type is represented in:

- [JSON-LD](https://stencila.org/Organization.jsonld)
- [JSON Schema](https://stencila.org/Organization.schema.json)
- Python class [`Organization`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/organization.py)
- Rust struct [`Organization`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/organization.rs)
- TypeScript class [`Organization`](https://github.com/stencila/stencila/blob/main/ts/src/types/Organization.ts)

# Source

This documentation was generated from [`Organization.yaml`](https://github.com/stencila/stencila/blob/main/schema/Organization.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
