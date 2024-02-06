// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

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
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for PersonOrOrganization: ${other.type}`);
  }
}
