# Person

The `Person` type allows you to provide details about a person such as their given and family names, any honorific prefix or suffix, and contact details such as an email address. This type of often used to describe the `authors` of an [`Article`](/Article), or other [`CreativeWork`](/CreativeWork).

## Examples

These examples, based on Marie Curie, illustrate alternative ways for specifying a `Person`. [Marie Curie](https://en.wikipedia.org/wiki/Marie_Curie) was the first woman to win a Nobel Prize, the first person and only woman to win twice, and the only person to win a Nobel Prize in two different sciences.

<!--
These examples will eventually be wrapped in React components
to illustrate how the input is converted into Stencila JSON
See https://github.com/stencila/schema/issues/45
-->

Dr Curie can be represented in canonical Stencila JSON by:

```json
{
  "type": "Person",
  "honorificPrefix": "Dr",
  "givenNames": ["Marie", "Skłodowska"],
  "familyNames": ["Curie"],
  "honorificSuffix": "PhD"
}
```

YAML provides a more readable format for providing details about a person in places like Markdown front-matter. In the following example, we take advantage of [property aliases](/docs/property-aliases) to use the shorter `prefix` and `suffix` property names, and the US convention of `firstNames` and `lastName` (instead of `givenNames` and `familyNames`). We also use [property parsing](/docs/property-parsing) to be be able to write `firstNames` as a space separated values.

```markdown
---
title: Recherches sur les substances radioactives
datePublished: 1904
authors:
  - prefix: Dr
    firstNames: Marie Skłodowska
    lastName: Curie
    suffix: PhD
---

Le présent travail a pour but d'exposer les recherches que je poursuis depuis plus de 4 ans sur les substances radioactives. J'ai commencé ces recherches par une étude du rayonnement uranique cjui a été découvert par M. Becquerel. Les résultats auxquels ...
```

We can shorten this further by specifying Dr Curie's details as a string:

```yaml
type: Article
title: Recherches sur les substances radioactives
datePublished: 1904
authors:
  - Dr Marie Skłodowska Curie PhD
```

If there had been email and web pages in the 1900s then we could also add those for her and her colleagues:

```yaml
type: Article
title: The Radioactive Constants as of 1930 Report of the International Radium-Standards Commission
year: 1931
url: https://link.aps.org/doi/10.1103/RevModPhys.3.427
authors:
  - Marie Curie <marie@mariecurie.org.uk> (https://www.mariecurie.org.uk/)
  - Debierne, A.
  - Eve, A. S.
  - Geiger, H.
  - Hahn, O.
  - Lind, S. C.
  - Meyer, St.
  - Rutherford, Ernest (https://en.wikipedia.org/wiki/Ernest_Rutherford)
  - Schweidler, E.
```

## Related

### JATS

`Person` is analogous, and structurally similar to, the JATS [`<contrib>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/contrib.html) element.
JATS includes `<custom-meta>` element, this _"element can be used as an escape-hatch to record additional metadata that a publisher or archive wishes to preserve (so that the intellectual work to capture that content will not be lost) even though there is no direct expression for the metadata in the Tag Set"._

### OpenDocument

`Person` is analogous to the OpenDocument `Author Fields` element which is the elements in a single OpenDocument XML document:

> The [`Author Fields`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415310_253892949) elements are:
>
> - The <text:author-name> element represents the full name of the author of a document.
> - The <text:author-initials> element represents the initials of the author of a document.

### HTML5

In HTML5 metadata about the author can be included in two ways.

1. If there is a link to the detailed information about the author (eg. their homepage), `rel="author"` with `link` or `a` [should be used](https://html.spec.whatwg.org/multipage/links.html#link-type-author).
   Example:

```html
<a href="http://johnsplace.com" rel="author">John</a>
```

2. If there is no link to the information about the author, `class="author"` attribute should be used with `area`, `span` and so on.
   Example:

```html
<span class="author">John</span>
```

### Crossref

`Person` is analogous to the Crossref [`<contributors>`](https://support.crossref.org/hc/en-us/articles/214567746-Authors-and-editors) child element
[`<person_name>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_person_name) which describes a person who contributed to authoring or editing an entity.

### Citation Style Language (CSL)

`Person` is analogous to the CSL JSON Schema for [`author`](https://github.com/citation-style-language/schema/blob/f01ba9c5ec2055e381a38598919a379255c496c5/csl-data.json#L72) items which describe a person who contributed to authoring an entity.

### ISA framework

The ISA metadata framework includes a [`person_schema.json`](https://isa-specs.readthedocs.io/en/latest/isajson.html#person-schema-json) which is similar to `Person`.

### Summary

The following table summarizes how properties of `Person` relate to other schema.

| `Person`         | Crossref `<person-name>` | CSL `author` | ISA person schema      | JATS `<contrib>` |
| ---------------- | ------------------------ | ------------ | ---------------------- | ---------------- |
| address          |                          |              | address                | address          |
| affiliation      | affiliation              |              | affiliation            | aff              |
| emails           |                          |              | email                  | email            |
| familyNames      | surname                  | family       | lastName               | surname          |
| funders          |                          |              |                        | funding-source   |
| givenNames       | given_name               | given        | firstName, midInitials | given-names      |
| honorificPrefix  |                          |              |                        | prefix           |
| honorificSuffix  | suffix                   | suffix       |                        | suffix, degrees  |
| jobTitle         |                          |              |                        | custom-meta      |
| memberOf         |                          |              |                        | custom-meta      |
| telephoneNumbers |                          |              | phone, fax             |                  |
