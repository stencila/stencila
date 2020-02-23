#' A person (alive, dead, undead, or fictional).
#'
#' @name Person
#' @param address Postal address for the person.
#' @param affiliations Organizations that the person is affiliated with.
#' @param alternateNames Alternate names (aliases) for the item.
#' @param description A description of the item.
#' @param emails Email addresses for the person.
#' @param familyNames Family name. In the U.S., the last name of a person.
#' @param funders A person or organization that supports (sponsors) something through some kind of financial contribution.
#' @param givenNames Given name. In the U.S., the first name of a person.
#' @param honorificPrefix An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
#' @param honorificSuffix An honorific suffix after a person's name such as MD/PhD/MSCSW.
#' @param id The identifier for this item.
#' @param identifiers Any kind of identifier for any kind of Thing.
#' @param images Images of the item.
#' @param jobTitle The job title of the person (for example, Financial Manager).
#' @param memberOf An organization (or program membership) to which this person belongs.
#' @param meta Metadata associated with this item.
#' @param name The name of the item.
#' @param telephoneNumbers Telephone numbers for the person.
#' @param url The URL of the item.
#' @seealso \code{\link{Thing}}
#' @export
Person <- function(
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
  identifiers,
  images,
  jobTitle,
  memberOf,
  meta,
  name,
  telephoneNumbers,
  url
){
  self <- Thing(
    alternateNames = alternateNames,
    description = description,
    id = id,
    identifiers = identifiers,
    images = images,
    meta = meta,
    name = name,
    url = url
  )
  self$type <- as_scalar("Person")
  self[["address"]] <- check_property("Person", "address", FALSE, missing(address), "character", address)
  self[["affiliations"]] <- check_property("Person", "affiliations", FALSE, missing(affiliations), Array(Organization), affiliations)
  self[["emails"]] <- check_property("Person", "emails", FALSE, missing(emails), Array("character"), emails)
  self[["familyNames"]] <- check_property("Person", "familyNames", FALSE, missing(familyNames), Array("character"), familyNames)
  self[["funders"]] <- check_property("Person", "funders", FALSE, missing(funders), Array(Union(Organization, Person)), funders)
  self[["givenNames"]] <- check_property("Person", "givenNames", FALSE, missing(givenNames), Array("character"), givenNames)
  self[["honorificPrefix"]] <- check_property("Person", "honorificPrefix", FALSE, missing(honorificPrefix), "character", honorificPrefix)
  self[["honorificSuffix"]] <- check_property("Person", "honorificSuffix", FALSE, missing(honorificSuffix), "character", honorificSuffix)
  self[["jobTitle"]] <- check_property("Person", "jobTitle", FALSE, missing(jobTitle), "character", jobTitle)
  self[["memberOf"]] <- check_property("Person", "memberOf", FALSE, missing(memberOf), Array(Organization), memberOf)
  self[["telephoneNumbers"]] <- check_property("Person", "telephoneNumbers", FALSE, missing(telephoneNumbers), Array("character"), telephoneNumbers)
  class(self) <- c(class(self), "Person")
  self
}

