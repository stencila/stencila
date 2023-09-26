// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Grant } from "./Grant.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";

/**
 * A monetary grant.
 */
export class MonetaryGrant extends Grant {
  type = "MonetaryGrant";

  /**
   * The amount of money.
   */
  amounts?: number;

  /**
   * A person or organization that supports (sponsors) something through some kind of financial contribution.
   */
  funders?: PersonOrOrganization[];

  constructor(options?: Partial<MonetaryGrant>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `MonetaryGrant`
*/
export function monetaryGrant(options?: Partial<MonetaryGrant>): MonetaryGrant {
  return new MonetaryGrant(options);
}
