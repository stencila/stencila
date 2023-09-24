// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Organization } from './Organization'
import { Person } from './Person'

// `Organization` or `Person`
export type OrganizationOrPerson =
  Organization |
  Person;

export function organizationOrPerson(other: OrganizationOrPerson): OrganizationOrPerson {
  switch(other.type) {
    case "Organization": return Organization.from(other as Organization);
    case "Person": return Person.from(other as Person);
    default: throw new Error(`Unexpected type for OrganizationOrPerson: ${other.type}`)
  }
}
