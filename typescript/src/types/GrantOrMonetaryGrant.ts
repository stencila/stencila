// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Grant } from "./Grant.js";
import { MonetaryGrant } from "./MonetaryGrant.js";

// `Grant` or `MonetaryGrant`
export type GrantOrMonetaryGrant =
  Grant |
  MonetaryGrant;

export function grantOrMonetaryGrantFrom(other: GrantOrMonetaryGrant): GrantOrMonetaryGrant {
  switch(other.type) {
    case "Grant": return Grant.from(other as Grant);
    case "MonetaryGrant": return MonetaryGrant.from(other as MonetaryGrant);
    default: throw new Error(`Unexpected type for GrantOrMonetaryGrant: ${other.type}`);
  }
}
