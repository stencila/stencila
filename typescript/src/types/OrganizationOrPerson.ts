// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";

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
    case "Organization":
    case "Person":
      return hydrate(other) as OrganizationOrPerson
    default:
      throw new Error(`Unexpected type for OrganizationOrPerson: ${other.type}`);
  }
}
