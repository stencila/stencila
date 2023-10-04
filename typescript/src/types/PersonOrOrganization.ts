// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";

/**
 * `Person` or `Organization`
 */
export type PersonOrOrganization =
  Person |
  Organization;

/**
 * Create a `PersonOrOrganization` from an object
 */
export function personOrOrganization(other: PersonOrOrganization): PersonOrOrganization {
  switch(other.type) {
    case "Person":
    case "Organization":
      return hydrate(other) as PersonOrOrganization
    default:
      throw new Error(`Unexpected type for PersonOrOrganization: ${other.type}`);
  }
}
