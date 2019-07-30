#` A person (alive, dead, undead, or fictional).
#`
#` @param  address Postal address for the person.
#` @param  affiliations Organizations that the person is affiliated with.
#` @param  alternateNames Alternate names (aliases) for the item.
#` @param  description A description of the item.
#` @param  emails Email addresses for the person.
#` @param  familyNames Family name. In the U.S., the last name of a person.
#` @param  funders A person or organization that supports (sponsors) something through some kind of financial contribution.
#` @param  givenNames Given name. In the U.S., the first name of a person.
#` @param  honorificPrefix An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
#` @param  honorificSuffix An honorific suffix after a person's name such as MD/PhD/MSCSW.
#` @param  id The identifier for this item.
#` @param  jobTitle The job title of the person (for example, Financial Manager).
#` @param  memberOf An organization (or program membership) to which this person belongs.
#` @param  meta Metadata associated with this item.
#` @param  name The name of the item.
#` @param  telephoneNumbers Telephone numbers for the person.
#` @param  url The URL of the item.
#` @export
Person <- function (
  address,
  affiliations,
  alternateNames,
  description,
  emails,
  familyNames,
  funders,
  givenNames,
  honorificPrefix,
  honorificSuffix,
  id,
  jobTitle,
  memberOf,
  meta,
  name,
  telephoneNumbers,
  url
){
  self <- Thing(
    alternateNames=alternateNames,
    description=description,
    id=id,
    meta=meta,
    name=name,
    url=url
  )
  if(!missing(address)) setProp(self, "address", "character", address)
  if(!missing(affiliations)) setProp(self, "affiliations", Array("Organization"), affiliations)
  if(!missing(emails)) setProp(self, "emails", Array("character"), emails)
  if(!missing(familyNames)) setProp(self, "familyNames", Array("character"), familyNames)
  if(!missing(funders)) setProp(self, "funders", Array(Union("Organization", "Person")), funders)
  if(!missing(givenNames)) setProp(self, "givenNames", Array("character"), givenNames)
  if(!missing(honorificPrefix)) setProp(self, "honorificPrefix", "character", honorificPrefix)
  if(!missing(honorificSuffix)) setProp(self, "honorificSuffix", "character", honorificSuffix)
  if(!missing(jobTitle)) setProp(self, "jobTitle", "character", jobTitle)
  if(!missing(memberOf)) setProp(self, "memberOf", Array("Organization"), memberOf)
  if(!missing(telephoneNumbers)) setProp(self, "telephoneNumbers", Array("character"), telephoneNumbers)
  class(self) <- c(class(self), "Person")
  self
}

