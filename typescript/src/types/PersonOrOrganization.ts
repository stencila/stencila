// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Organization } from "./Organization.js";
import { Person } from "./Person.js";

// `Person` or `Organization`
export type PersonOrOrganization =
  Person |
  Organization;

export function personOrOrganizationFrom(other: PersonOrOrganization): PersonOrOrganization {
  switch(other.type) {
    case "Person": return Person.from(other as Person);
    case "Organization": return Organization.from(other as Organization);
    default: throw new Error(`Unexpected type for PersonOrOrganization: ${other.type}`);
  }
}
