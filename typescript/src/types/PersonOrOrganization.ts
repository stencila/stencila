// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Organization } from './Organization'
import { Person } from './Person'

// `Person` or `Organization`
export type PersonOrOrganization =
  Person |
  Organization;

export function personOrOrganization(other: PersonOrOrganization): PersonOrOrganization {
  switch(other.type) {
    case "Person": return Person.from(other as Person);
    case "Organization": return Organization.from(other as Organization);
    default: throw new Error(`Unexpected type for PersonOrOrganization: ${other.type}`)
  }
}
