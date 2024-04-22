// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type Thing } from "./Thing.js";

/**
 * Union type for things that can be an author in `AuthorRole`.
 */
export type AuthorRoleAuthor =
  Person |
  Organization |
  SoftwareApplication |
  Thing;

/**
 * Create a `AuthorRoleAuthor` from an object
 */
export function authorRoleAuthor(other: AuthorRoleAuthor): AuthorRoleAuthor {
  switch(other.type) {
    case "Person":
    case "Organization":
    case "SoftwareApplication":
    case "Thing":
      return hydrate(other) as AuthorRoleAuthor
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for AuthorRoleAuthor: ${other.type}`);
  }
}
