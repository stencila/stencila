# `Person`

## Extends

`Person` extends `Thing`.

## Related

### JATS

`Person` is analagous, and structurally similar to, the JATS
[`<contrib>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/contrib.html) element:

> A journal author `<contrib>` links to a number of JATS elements which
> are also properties of `Person` in Stencila schema. These JATS elements are:

| Stencila Schema | JATS            |
|-----------------|-----------------|
| familyNames     | surname         |
| givenNames      | given-names     |
| emails          | email           |
| affiliations    | aff             |
| brand           | custom-meta     |
| contactPoint    | corresp         |
| funders         | funding-source  |
| honorificPrefix | prefix          |
| honorificSuffix | suffix, degrees |
| jobTitle        | custom-meta     |
| memberOf        | custom-meta     |

**Note** JATS includes `<custom-meta>` element
_This element can be used as an escape-hatch to record additional metadata that a publisher or archive wishes to preserve (so that the intellectual work to capture that content will not be lost) even though there is no direct expression for the metadata in the Tag Set._

### OpenDocument

`Person` is analagous to the OpenDocument `Author Fields` element which is the elements in a single* OpenDocument XML document:

> The [`Author Fields`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415310_253892949) elements are: 
> - The <text:author-name> element represents the full name of the author of a document.
> - The <text:author-initials> element represents the initials of the author of a document.


### HTML5

In HTML5 metadata about the author can be included in two ways.

1. If there is a link to the detailed information about the author (eg. their homepage), `rel="author"` with `link` or `a` [should be used](https://html.spec.whatwg.org/multipage/links.html#link-type-author).
   Example:

```
<a href="http://johnsplace.com" rel="author">John</a>
```

2. If there is no link to the information about the author, `class="author"` attribute should be used with `area`, `span` and so on.
   Example:

```
<span class="author">John</span>
```

### Crossref

`Person` is analogous to the Crossref Contributors (`<crossref:contributors>`) child element 
[`<person_name>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_person_name) which describes a person who contributed to authoring or editing an entity. `<contributors:person-name>` has a number of child elements:

| Stencila Schema | `<contributors:person-name>` |
|-----------------|------------------------------|
| givenNames      | given_name                   |
| familyNames     | surname                      |
| honorificSuffix | suffix                       |
| affiliation     | affiliation                  |



### Citation Style Language (CSL)

`Person` is analogous to the JSON schema for CSL input data property [`<author>`](https://raw.githubusercontent.com/citation-style-language/schema/master/csl-data.json) which describes a person who contributed to authoring an entity. `<author>` has a number of child elements:

| Stencila Schema | `<author>` |
|-----------------|------------|
| familyNames     | family     |
| givenNames      | given      |
| honorificSuffix | suffix     |
