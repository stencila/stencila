---
title: Person
description: A person (alive, dead, undead, or fictional).
config:
  publish:
    ghost:
      type: post
      slug: person
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `Person` type has these properties:

| Name               | Description                                                                                              | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                                  | Aliases                                                                                                                    |
| ------------------ | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `id`               | The identifier for this item.                                                                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                           | -                                                                                                                          |
| `alternateNames`   | Alternate names (aliases) for the item.                                                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName)     | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name`                                  |
| `description`      | A description of the item.                                                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)         | -                                                                                                                          |
| `identifiers`      | Any kind of identifier for any kind of Thing.                                                            | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)           | `identifier`                                                                                                               |
| `images`           | Images of the item.                                                                                      | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                     | `image`                                                                                                                    |
| `name`             | The name of the item.                                                                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                       | -                                                                                                                          |
| `url`              | The URL of the item.                                                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                         | -                                                                                                                          |
| `orcid`            | The person's Open Researcher and Contributor ID (https://orcid.org/).                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | `stencila:orcid`                                               | -                                                                                                                          |
| `address`          | Postal address for the person.                                                                           | [`PostalAddress`](https://stencila.ghost.io/docs/reference/schema/postal-address) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)    | -                                                                  | [`schema:address`](https://schema.org/address)                 | -                                                                                                                          |
| `affiliations`     | Organizations that the person is affiliated with.                                                        | [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization)*                                                                            | -                                                                  | [`schema:affiliation`](https://schema.org/affiliation)         | `affiliation`                                                                                                              |
| `emails`           | Email addresses for the person.                                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:email`](https://schema.org/email)                     | `email`                                                                                                                    |
| `familyNames`      | Family name. In the U.S., the last name of a person.                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:familyName`](https://schema.org/familyName)           | `familyName`, `surname`, `surnames`, `lastName`, `lastNames`, `family-names`, `family_names`, `family-name`, `family_name` |
| `funders`          | A person or organization that supports (sponsors) something through some kind of financial contribution. | ([`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization))*    | -                                                                  | [`schema:funder`](https://schema.org/funder)                   | `funder`                                                                                                                   |
| `givenNames`       | Given name. In the U.S., the first name of a person.                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:givenName`](https://schema.org/givenName)             | `firstName`, `firstNames`, `given-names`, `given_names`, `givenName`, `given-name`, `given_name`                           |
| `honorificPrefix`  | An honorific prefix preceding a person's name such as Dr/Mrs/Mr.                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:honorificPrefix`](https://schema.org/honorificPrefix) | `prefix`, `honorific-prefix`, `honorific_prefix`                                                                           |
| `honorificSuffix`  | An honorific suffix after a person's name such as MD/PhD/MSCSW.                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:honorificSuffix`](https://schema.org/honorificSuffix) | `suffix`, `honorific-suffix`, `honorific_suffix`                                                                           |
| `jobTitle`         | The job title of the person (for example, Financial Manager).                                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:jobTitle`](https://schema.org/jobTitle)               | `job-title`, `job_title`                                                                                                   |
| `memberOf`         | An organization (or program membership) to which this person belongs.                                    | [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization)*                                                                            | -                                                                  | [`schema:memberOf`](https://schema.org/memberOf)               | `member-of`, `member_of`                                                                                                   |
| `telephoneNumbers` | Telephone numbers for the person.                                                                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:telephone`](https://schema.org/telephone)             | `telephone`, `telephone-numbers`, `telephone_numbers`, `telephoneNumber`, `telephone-number`, `telephone_number`           |

# Related

The `Person` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: none

# Formats

The `Person` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |         |

# Bindings

The `Person` type is represented in:

- [JSON-LD](https://stencila.org/Person.jsonld)
- [JSON Schema](https://stencila.org/Person.schema.json)
- Python class [`Person`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/person.py)
- Rust struct [`Person`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/person.rs)
- TypeScript class [`Person`](https://github.com/stencila/stencila/blob/main/ts/src/types/Person.ts)

# Source

This documentation was generated from [`Person.yaml`](https://github.com/stencila/stencila/blob/main/schema/Person.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
