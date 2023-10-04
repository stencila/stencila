// Generated file; do not edit. See `../rust/schema-gen` crate.

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
      throw new Error(`Unexpected type for GrantOrMonetaryGrant: ${other.type}`);
  }
}
