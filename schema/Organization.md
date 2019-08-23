---
title: Organization
authors: []
---

include: ../public/Organization.schema.md
:::
An organization such as a school, NGO, corporation, club, etc. https&#x3A;//schema.org/Organization.

| Entity       | type               | The name of the type and all descendant types.                                                                | string |
| ------------ | ------------------ | ------------------------------------------------------------------------------------------------------------- | ------ |
| Entity       | id                 | The identifier for this item.                                                                                 | string |
| Thing        | alternateNames     | Alternate names (aliases) for the item.                                                                       | array  |
| Thing        | description        | A description of the item.                                                                                    | string |
| Thing        | meta               | Metadata associated with this item.                                                                           | object |
| Thing        | name               | The name of the item.                                                                                         | string |
| Thing        | url                | The URL of the item.                                                                                          | string |
| Organization | address            | Postal address for the organization.                                                                          |        |
| string       |                    |                                                                                                               |        |
| Organization | brands             | Brands that the organization is connected with.                                                               |        |
| array        |                    |                                                                                                               |        |
| Organization | contactPoints      | Correspondence/Contact points for the organization.                                                           |        |
| array        |                    |                                                                                                               |        |
| Organization | departments        | Departments within the organization. For example, Department of Computer Science, Research & Development etc. |        |
| array        |                    |                                                                                                               |        |
| Organization | funders            | Organization(s) or person(s) funding the organization.                                                        |        |
| array        |                    |                                                                                                               |        |
| Organization | legalName          | Legal name for the Organization. Should only include letters and spaces.                                      |        |
| string       |                    |                                                                                                               |        |
| Organization | parentOrganization | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.   |        |
|              |                    |                                                                                                               |        |

:::

The `Organization` type allows you to provide details about an organization such as its legal name and departments, contact details: address, email and telephone as well as information about its funders. This type of often used to describe the `affiliations` of a [`Person`](/Person).

# Examples

The example below are based on the University of Otago and show different ways of specifying an `Organization`. [The University of Otago](https://www.otago.ac.nz/) is the oldest university in Aotearoa New Zealand.

&lt;!-- These examples will eventually be wrapped in React components to illustrate how the input is converted into Stencila JSON See https&#x3A;//github.com/stencila/schema/issues/45 -->

The University of Otago can be represented in canonical Stencila JSON by:

```json import=example
{
  "type": "Organization",
  "address": "362 Leith Street, Dunedin 9054, New Zealand",
  "brands": [
    {
      "type": "Brand",
      "logo": "http://www.otago.ac.nz/logo"
    }
  ],
  "contactPoints": [
    {
      "availableLanguages": ["English", "Māori"],
      "emails": ["office@otago.ac.nz"],
      "telephone": "00641234567"
    }
  ],
  "departments": [
    { "type": "Organization", "legalName": "Commerce" },
    { "type": "Organization", "legalName": "Health_Sciences" },
    { "type": "Organization", "legalName": "Humanities" }
  ],
  "funders": [
    { "type": "Organization", "legalName": "MBIE" },
    { "type": "Organization", "legalName": "CancerSociety" }
  ],
  "legalName": "The University of Otago"
}
```

The schema allows for an `Organization` to have a `parentOrganization`. For example, `parentOrganization` to a department is a university.

```json import=example_with_parent
{
  "type": "Organization",
  "address": "Sciences Building, Dunedin, New Zealand",
  "legalName": "Department of Natural Sciences",
  "parentOrganization": {
    "type": "Organization",
    "legalName": "The University of Otago"
  }
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`Organization` is analogous to the JATS [`<institution-wrap>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/institution-wrap.html) element which is a wrapper element to hold both the name of an institution (`<institution>`) and any identifiers for that institution (`<institution-id>`).

```jats

```

```jats

```

## OpenDocument

`Organization` is analogous to the Open Document [`<text:organizations>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1419060_253892949) and [`<text:institutions>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1418948_253892949) attributes.

## Crossref

`Organization` is analogous to the Crossref Organization element [`<crossref:organization>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_organization) which is the name of an organization (as opposed to a person) that contributed to authoring an entity. If multiple organizations authored an entity, each one should be captured in a unique organization element.

[//]: # 'WIP: Needs JATS Fixes'
