## Related

### JATS

`Organization` is analagous to the JATS
[`<institution-wrap>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/institution-wrap.html) element
which is a wrapper element to hold both the name of an institution (`<institution>`) and any identifiers for that institution (`<institution-id>`)
Example:

```
<institution-wrap>
  <institution-id>Moo-U-41</institution-id>
  <institution content-type="edu">
  University of Frostbite Falls, Dept of Campus Security, 
  Dept of Moose and Squirrel Security, 
  Office of the Acting Dean</institution>
</institution-wrap>
```

### OpenDocument

`Organization` is analogous to the Open Document [`<text:organizations>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1419060_253892949)
 and [`<text:institutions>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1418948_253892949) attributes.


### Crossref

`Organization` is analogous to the Crossref Organization element [`<crossref:organization>`](https://data.crossref.org/reports/help/schema_doc/4.4.0/relations_xsd.html#http___www.crossref.org_relations.xsd_organization) which is the name of an organization (as opposed to a person) that contributed to authoring an entity. If multiple organizations authored an entity, each one should be captured in a unique organization element.


### Citation Style Language (CSL)

At the moment there does not seem to be a schema element in CSL that `Organization` is analogous to. If you are able to add more information about this, please [edit this file](https://github.com/stencila/schema/edit/master/schema/Organization.schema.yaml).


### HTML5

At the moment there does not seem to be an HTML5 tag or attribute that `Organization` is analogous to. However, it may be possible to [use link types](http://w3c.github.io/html/links.html#sec-link-types). If you are able to add more information about this, please [edit this file](https://github.com/stencila/schema/edit/master/schema/Organization.schema.yaml).