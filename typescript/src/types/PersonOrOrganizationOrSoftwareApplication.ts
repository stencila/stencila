// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Organization } from "./Organization.js";
import { Person } from "./Person.js";
import { SoftwareApplication } from "./SoftwareApplication.js";

/**
 * `Person` or `Organization` or `SoftwareApplication`
 */
export type PersonOrOrganizationOrSoftwareApplication =
  Person |
  Organization |
  SoftwareApplication;

/**
 * Create a `PersonOrOrganizationOrSoftwareApplication` from an object
 */
export function personOrOrganizationOrSoftwareApplication(other: PersonOrOrganizationOrSoftwareApplication): PersonOrOrganizationOrSoftwareApplication {
  switch(other.type) {
    case "Person":
    case "Organization":
    case "SoftwareApplication":
      return hydrate(other) as PersonOrOrganizationOrSoftwareApplication
    default:
      throw new Error(`Unexpected type for PersonOrOrganizationOrSoftwareApplication: ${other.type}`);
  }
}
