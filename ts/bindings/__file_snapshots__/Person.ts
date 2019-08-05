/**
 * A person (alive, dead, undead, or fictional).
 */
export interface Person extends Thing {
  type: 'Person'
  address?: string
  affiliations?: Organization[]
  emails?: string[]
  familyNames?: string[]
  funders?: (Organization | Person)[]
  givenNames?: string[]
  honorificPrefix?: string
  honorificSuffix?: string
  jobTitle?: string
  memberOf?: Organization[]
  telephoneNumbers?: string[]
}

/**
 * Create a `Person` node
 * @param options Optional properties
 * @returns {Person}
 */
export const person = (options: OptionalProps<Person> = {}): Person => ({
  ...compact(options),
  type: 'Person'
})
