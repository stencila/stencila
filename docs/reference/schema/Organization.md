# Organization

**An organization such as a school, NGO, corporation, club, etc.**

This is an implementation of schema.org [`Organization`](https://schema.org/Organization). An `Organization` node is analogous, and structurally similar to: - Crossref Organization element [`<crossref:organization>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_organization) - JATS XML [`<institution-wrap>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/institution-wrap.html) - Open Document [`<text:organizations>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1419060_253892949) and [`<text:institutions>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1418948_253892949)

## Properties

| Name               | `@id`                                                              | Type                                                                                                 | Description                                                                                                   | Inherited from                  |
| ------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| address            | [schema:address](https://schema.org/address)                       | [PostalAddress](PostalAddress.md) _or_ string                                                        | Postal address for the organization.                                                                          | [Organization](Organization.md) |
| alternateNames     | [schema:alternateName](https://schema.org/alternateName)           | Array of string                                                                                      | Alternate names (aliases) for the item.                                                                       | [Thing](Thing.md)               |
| brands             | [schema:brand](https://schema.org/brand)                           | Array of [Brand](Brand.md)                                                                           | Brands that the organization is connected with.                                                               | [Organization](Organization.md) |
| contactPoints      | [schema:contactPoint](https://schema.org/contactPoint)             | Array of [ContactPoint](ContactPoint.md)                                                             | Correspondence/Contact points for the organization.                                                           | [Organization](Organization.md) |
| departments        | [schema:department](https://schema.org/department)                 | Array of [Organization](Organization.md)                                                             | Departments within the organization. For example, Department of Computer Science, Research & Development etc. | [Organization](Organization.md) |
| description        | [schema:description](https://schema.org/description)               | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [1](#notes).                                                              | [Thing](Thing.md)               |
| funders            | [schema:funder](https://schema.org/funder)                         | Array of ([Organization](Organization.md) _or_ [Person](Person.md))                                  | Organization(s) or person(s) funding the organization.                                                        | [Organization](Organization.md) |
| id                 | [schema:id](https://schema.org/id)                                 | string                                                                                               | The identifier for this item.                                                                                 | [Entity](Entity.md)             |
| identifiers        | [schema:identifier](https://schema.org/identifier)                 | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [2](#notes).                                           | [Thing](Thing.md)               |
| images             | [schema:image](https://schema.org/image)                           | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                                                           | [Thing](Thing.md)               |
| legalName          | [schema:legalName](https://schema.org/legalName)                   | string                                                                                               | Legal name for the Organization. Should only include letters and spaces.                                      | [Organization](Organization.md) |
| logo               | [schema:logo](https://schema.org/logo)                             | [ImageObject](ImageObject.md) _or_ Format 'uri'                                                      | The logo of the organization. See note [3](#notes).                                                           | [Organization](Organization.md) |
| members            | [schema:member](https://schema.org/member)                         | Array of ([Organization](Organization.md) _or_ [Person](Person.md))                                  | Person(s) or organization(s) who are members of this organization.                                            | [Organization](Organization.md) |
| meta               | [stencila:meta](https://schema.stenci.la/meta.jsonld)              | object                                                                                               | Metadata associated with this item.                                                                           | [Entity](Entity.md)             |
| name               | [schema:name](https://schema.org/name)                             | string                                                                                               | The name of the item.                                                                                         | [Thing](Thing.md)               |
| parentOrganization | [schema:parentOrganization](https://schema.org/parentOrganization) | [Organization](Organization.md)                                                                      | Entity that the Organization is a part of. For example, parentOrganization to a department is a university.   | [Organization](Organization.md) |
| url                | [schema:url](https://schema.org/url)                               | Format 'uri'                                                                                         | The URL of the item.                                                                                          | [Thing](Thing.md)               |

## Notes

1. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
2. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).
3. **logo** : This is a singleton property because, at any one time, an organization will usually only have one logo.

## Examples

```json
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

```json
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
      "type": "ContactPoint",
      "availableLanguages": ["English", "Māori"],
      "emails": ["office@otago.ac.nz"],
      "telephone": "00641234567"
    }
  ],
  "departments": [
    {
      "type": "Organization",
      "legalName": "Commerce"
    },
    {
      "type": "Organization",
      "legalName": "Health_Sciences"
    },
    {
      "type": "Organization",
      "legalName": "Humanities"
    }
  ],
  "funders": [
    {
      "type": "Organization",
      "legalName": "MBIE"
    },
    {
      "type": "Organization",
      "legalName": "CancerSociety"
    }
  ],
  "legalName": "The University of Otago"
}
```

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Organization.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Organization.schema.json)
- Python [`class Organization`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Organization)
- TypeScript [`interface Organization`](https://stencila.github.io/schema/ts/docs/interfaces/organization.html)
- R [`class Organization`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Organization`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Organization.html)

## Source

This documentation was generated from [Organization.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Organization.schema.yaml).
