/**
 * A person (alive, dead, undead, or fictional). https://schema.org/Person.
 */
export interface Person extends Thing {
  type: 'Person'
  address?: string
  affiliations?: Array<Organization>
  emails?: Array<string>
  familyNames?: Array<string>
  funders?: Array<Organization | Person>
  givenNames?: Array<string>
  honorificPrefix?: string
  honorificSuffix?: string
  jobTitle?: string
  memberOf?: Array<Organization>
  telephoneNumbers?: Array<string>
}

/**
 * Create a `Person` node
 * @param options Optional properties
 * @returns {Person}
 */
export const person = (
  options: {
    address?: string,
    affiliations?: Array<Organization>,
    alternateNames?: Array<string>,
    description?: string,
    emails?: Array<string>,
    familyNames?: Array<string>,
    funders?: Array<Organization | Person>,
    givenNames?: Array<string>,
    honorificPrefix?: string,
    honorificSuffix?: string,
    id?: string,
    jobTitle?: string,
    memberOf?: Array<Organization>,
    meta?: {[key: string]: any},
    name?: string,
    telephoneNumbers?: Array<string>,
    url?: string
  } = {}
): Person => ({
  ...options,
  type: 'Person'
})

