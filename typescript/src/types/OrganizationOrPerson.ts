// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Organization } from "./Organization.js";
import { Person } from "./Person.js";

/**
 * `Organization` or `Person`
 */
export type OrganizationOrPerson =
  Organization |
  Person;

/**
 * Create a `OrganizationOrPerson` from an object
 */
export function organizationOrPerson(other: OrganizationOrPerson): OrganizationOrPerson {
  switch(other.type) {
    case "Organization": return Organization.from(other as Organization);
    case "Person": return Person.from(other as Person);
    default: throw new Error(`Unexpected type for OrganizationOrPerson: ${other.type}`);
  }
}
