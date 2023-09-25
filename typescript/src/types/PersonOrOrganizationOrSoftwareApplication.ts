// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Organization } from "./Organization.js";
import { Person } from "./Person.js";
import { SoftwareApplication } from "./SoftwareApplication.js";

// `Person` or `Organization` or `SoftwareApplication`
export type PersonOrOrganizationOrSoftwareApplication =
  Person |
  Organization |
  SoftwareApplication;

export function personOrOrganizationOrSoftwareApplicationFrom(other: PersonOrOrganizationOrSoftwareApplication): PersonOrOrganizationOrSoftwareApplication {
  switch(other.type) {
    case "Person": return Person.from(other as Person);
    case "Organization": return Organization.from(other as Organization);
    case "SoftwareApplication": return SoftwareApplication.from(other as SoftwareApplication);
    default: throw new Error(`Unexpected type for PersonOrOrganizationOrSoftwareApplication: ${other.type}`);
  }
}
