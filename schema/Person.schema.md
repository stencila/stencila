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
| --------------- | --------------- |
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
| owns            | product         |

**Note** JATS includes `<custom-meta>` element
_This element can be used as an escape-hatch to record additional metadata that a publisher or archive wishes to preserve (so that the intellectual work to capture that content will not be lost) even though there is no direct expression for the metadata in the Tag Set._

### OpenDocument

http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415310_253892949

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
