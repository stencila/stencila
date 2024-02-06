// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";

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
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for PersonOrOrganizationOrSoftwareApplication: ${other.type}`);
  }
}
