# Organization

The `Organization` type allows you to provide details about an organization such as its legal name and departments, contact details: address, email and telephone as well as information about its funders. This type of often used to describe the `affiliations` of a [`Person`](/Person).

## Examples
The examples below are based on the University of Otago and show different ways of specifying an `Organization`. [The University of Otago](https://www.otago.ac.nz/) is the oldest university in Aotearoa New Zealand. 

<!--
These examples will eventually be wrapped in React components
to illustrate how the input is converted into Stencila JSON
See https://github.com/stencila/schema/issues/45
-->

The University of Otago can be represented in canonical Stencila JSON by:


```json
{
  "type": "Organization",
  "address": "362 Leith Street, Dunedin 9054, New Zealand",
  "brands": "Otago",
  "contactPoints": {
    "availableLanguages": ["English", "Māori"],
    "emails": ["office@otago.ac.nz"],
    "telephone": "00641234567",
  },
  "departments": [Commerce.yaml, Health_Sciences.yaml, Humanities.yaml],
  "funders": [ MBIE.yaml, CancerSociety.yaml],
  "legalName": "The University of Otago"
}
```

```markdown
---
title: Introduction to Computer Science
authors:
  - prefix: Dr
    firstNames: Jane
    lastName: Smith
    suffix: PhD
    affiliations: University of Otago
      - address: 362 Leith Street, Dunedin 9054, New Zealand
        brands: Otago
        contactPoints: 
           - languages: English, Māori
             emails: office@otago.ac.nz
             telephone: 00641234567
        departments: Department of Computer Science
        legalName: The University of Otago
---

```

We can shorten this by specifying author's affiliation details as a string:

```yaml
type: Book
title: Introduction to Computer Science
authors:
   - Dr Jane Smith <jane.smith@otago.ac.nz> Department of Computer Science The University of Otago
```

## Related

### JATS

`Organization` is analagous to the JATS
[`<institution-wrap>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/institution-wrap.html) element
which is a wrapper element to hold both the name of an institution (`<institution>`) and any identifiers for that institution (`<institution-id>`).

JATS includes `<custom-meta>` element, this _"element can be used as an escape-hatch to record additional metadata that a publisher or archive wishes to preserve (so that the intellectual work to capture that content will not be lost) even though there is no direct expression for the metadata in the Tag Set"._

Example:

```
<institution-wrap>
  <institution-id>Moo-U-41</institution-id>
  <institution content-type="edu">
  The University of Otago</institution>
  <institution content-type="brand">
  Otago</institution>
  <institution content-type="dept">
  Department of Computer Science </institution>
  <institution content-type="contact-language">
  English, Māori </institution>
  <institution content-type="contact-email">
  office@otago.ac.nz </institution>
  <institution content-type="contact-phone">
  00641234567 </institution>
</institution-wrap>
```

Note that `<institution>` in JATS is an adrress class element

```
<contrib contrib-type="author">
<name><surname>Smith</surname>
<given-names>Jane</given-names></name>
<degrees>Ph D</degrees>
<aff>The University of Otago</aff>
<address>
<institution content-type="edu">The University of Otago</institution>
<institution content-type="dept">Department of Computer Science </institution>
<addr-line>362 Leith Street</addr-line>
<addr-line> Dunedin 9054/addr-line>
<country>New Zealand</country>
<email>jane.smith@otago.ac.nz</email>
</address>
</contrib>
```

### OpenDocument

`Organization` is analogous to the Open Document [`<text:organizations>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1419060_253892949)
and [`<text:institutions>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1418948_253892949) attributes.

### HTML5

At the moment there does not seem to be an HTML5 tag or attribute that `Organization` is analogous to. However, it may be possible to [use link types](http://w3c.github.io/html/links.html#sec-link-types). If you are able to add more information about this, please [edit this file](https://github.com/stencila/schema/edit/master/schema/Organization.schema.yaml).


### Crossref

`Organization` is analogous to the Crossref Organization element [`<crossref:organization>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_organization) which is the name of an organization (as opposed to a person) that contributed to authoring an entity. If multiple organizations authored an entity, each one should be captured in a unique organization element.

### Citation Style Language (CSL)

At the moment there does not seem to be a schema element in CSL that `Organization` is analogous to. If you are able to add more information about this, please [edit this file](https://github.com/stencila/schema/edit/master/schema/Organization.schema.yaml).


### ISA framework

The ISA metadata framework does not include a schema analogous to `Organization`. However, the [`person_schema.json`](https://isa-specs.readthedocs.io/en/latest/isajson.html#person-schema-json) includes elements corresponding to the elements in `Organization` in Stencila Schema.

Example:

```json
{
    "$schema": "http://json-schema.org/draft-04/schema",
    "title" : "ISA person schema",
    "type" : "object",
    "properties" : {
        "lastName" : { "Smith" },
        "firstName" : { "Jane" },
        "email" : { "jane.smith@otago.ac.nz" },
        "address" : { "362 Leith Street, Dunedin 9054, New Zealand" },
        "affiliation" : { "The University of Otago" },
}
```



