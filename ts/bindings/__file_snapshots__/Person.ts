/**
 * A person (alive, dead, undead, or fictional).
 */
export interface Person extends Thing {
  type: 'Person'
  address?: PostalAddress | string
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
 * @param props Object containing Person schema properties as key/value pairs
 * @returns {Person} Person schema node
 */
export const person = (
  props: Omit<Person, 'type'> = {}
): Person => ({
  ...compact(props),
  type: 'Person'
})

