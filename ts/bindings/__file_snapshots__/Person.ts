/**
 * A person (alive, dead, undead, or fictional).
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
  options: OptionalProps<Person> = {}
): Person => ({
  ...(compact(options)),
  type: 'Person'
})

