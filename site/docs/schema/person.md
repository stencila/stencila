---
title: Person
description: A person (alive, dead, undead, or fictional).
---

# Properties

The `Person` type has these properties:

| Name               | Description                                                                                              | Type                                                                 | Inherited from          | `JSON-LD @id`                                                  | Aliases                                                                                                                    |
| ------------------ | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `id`               | The identifier for this item.                                                                            | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                           | -                                                                                                                          |
| `alternateNames`   | Alternate names (aliases) for the item.                                                                  | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName)     | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name`                                  |
| `description`      | A description of the item.                                                                               | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)         | -                                                                                                                          |
| `identifiers`      | Any kind of identifier for any kind of Thing.                                                            | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)           | `identifier`                                                                                                               |
| `images`           | Images of the item.                                                                                      | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                     | `image`                                                                                                                    |
| `name`             | The name of the item.                                                                                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                       | -                                                                                                                          |
| `url`              | The URL of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                         | -                                                                                                                          |
| `orcid`            | The person's Open Researcher and Contributor ID (https://orcid.org/).                                    | [`String`](./string.md)                                              | -                       | `stencila:orcid`                                               | -                                                                                                                          |
| `address`          | Postal address for the person.                                                                           | [`PostalAddress`](./postal-address.md) \| [`String`](./string.md)    | -                       | [`schema:address`](https://schema.org/address)                 | -                                                                                                                          |
| `affiliations`     | Organizations that the person is affiliated with.                                                        | [`Organization`](./organization.md)*                                 | -                       | [`schema:affiliation`](https://schema.org/affiliation)         | `affiliation`                                                                                                              |
| `emails`           | Email addresses for the person.                                                                          | [`String`](./string.md)*                                             | -                       | [`schema:email`](https://schema.org/email)                     | `email`                                                                                                                    |
| `familyNames`      | Family name. In the U.S., the last name of a person.                                                     | [`String`](./string.md)*                                             | -                       | [`schema:familyName`](https://schema.org/familyName)           | `familyName`, `surname`, `surnames`, `lastName`, `lastNames`, `family-names`, `family_names`, `family-name`, `family_name` |
| `funders`          | A person or organization that supports (sponsors) something through some kind of financial contribution. | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       | [`schema:funder`](https://schema.org/funder)                   | `funder`                                                                                                                   |
| `givenNames`       | Given name. In the U.S., the first name of a person.                                                     | [`String`](./string.md)*                                             | -                       | [`schema:givenName`](https://schema.org/givenName)             | `firstName`, `firstNames`, `given-names`, `given_names`, `givenName`, `given-name`, `given_name`                           |
| `honorificPrefix`  | An honorific prefix preceding a person's name such as Dr/Mrs/Mr.                                         | [`String`](./string.md)                                              | -                       | [`schema:honorificPrefix`](https://schema.org/honorificPrefix) | `prefix`, `honorific-prefix`, `honorific_prefix`                                                                           |
| `honorificSuffix`  | An honorific suffix after a person's name such as MD/PhD/MSCSW.                                          | [`String`](./string.md)                                              | -                       | [`schema:honorificSuffix`](https://schema.org/honorificSuffix) | `suffix`, `honorific-suffix`, `honorific_suffix`                                                                           |
| `jobTitle`         | The job title of the person (for example, Financial Manager).                                            | [`String`](./string.md)                                              | -                       | [`schema:jobTitle`](https://schema.org/jobTitle)               | `job-title`, `job_title`                                                                                                   |
| `memberOf`         | An organization (or program membership) to which this person belongs.                                    | [`Organization`](./organization.md)*                                 | -                       | [`schema:memberOf`](https://schema.org/memberOf)               | `member-of`, `member_of`                                                                                                   |
| `telephoneNumbers` | Telephone numbers for the person.                                                                        | [`String`](./string.md)*                                             | -                       | [`schema:telephone`](https://schema.org/telephone)             | `telephone`, `telephone-numbers`, `telephone_numbers`, `telephoneNumber`, `telephone-number`, `telephone_number`           |

# Related

The `Person` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Formats

The `Person` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |         |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |         |
| [JATS](../formats/jats.md)                       |              |              |         |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |         |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |         |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |         |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |         |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |         |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](../formats/csl.md)                    |              |              |         |
| [Citation File Format](../formats/cff.md)        |              |              |         |
| [CSV](../formats/csv.md)                         |              |              |         |
| [TSV](../formats/tsv.md)                         |              |              |         |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |         |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |         |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |         |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |         |
| [Directory](../formats/directory.md)             |              |              |         |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |         |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |         |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |         |
| [Email HTML](../formats/email.html.md)           |              |              |         |
| [MJML](../formats/mjml.md)                       |              |              |         |

# Bindings

The `Person` type is represented in:

- [JSON-LD](https://stencila.org/Person.jsonld)
- [JSON Schema](https://stencila.org/Person.schema.json)
- Python class [`Person`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/person.py)
- Rust struct [`Person`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/person.rs)
- TypeScript class [`Person`](https://github.com/stencila/stencila/blob/main/ts/src/types/Person.ts)

# Source

This documentation was generated from [`Person.yaml`](https://github.com/stencila/stencila/blob/main/schema/Person.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
