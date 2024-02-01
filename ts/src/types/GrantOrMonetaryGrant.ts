// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Grant } from "./Grant.js";
import { type MonetaryGrant } from "./MonetaryGrant.js";

/**
 * `Grant` or `MonetaryGrant`
 */
export type GrantOrMonetaryGrant =
  Grant |
  MonetaryGrant;

/**
 * Create a `GrantOrMonetaryGrant` from an object
 */
export function grantOrMonetaryGrant(other: GrantOrMonetaryGrant): GrantOrMonetaryGrant {
  switch(other.type) {
    case "Grant":
    case "MonetaryGrant":
      return hydrate(other) as GrantOrMonetaryGrant
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for GrantOrMonetaryGrant: ${other.type}`);
  }
}
