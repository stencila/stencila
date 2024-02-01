// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type AuthorRole } from "./AuthorRole.js";
import { type Organization } from "./Organization.js";
import { type Person } from "./Person.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";

/**
 * Union type for things that can be an author of a `CreativeWork` or other type.
 */
export type Author =
  Person |
  Organization |
  SoftwareApplication |
  AuthorRole;

/**
 * Create a `Author` from an object
 */
export function author(other: Author): Author {
  switch(other.type) {
    case "Person":
    case "Organization":
    case "SoftwareApplication":
    case "AuthorRole":
      return hydrate(other) as Author
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for Author: ${other.type}`);
  }
}
