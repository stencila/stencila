// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Grant } from './Grant';
import { PersonOrOrganization } from './PersonOrOrganization';

// A monetary grant.
export class MonetaryGrant extends Grant {
  type = "MonetaryGrant";

  // The amount of money.
  amounts?: number;

  // A person or organization that supports (sponsors) something through some kind of financial contribution.
  funders?: PersonOrOrganization[];

  constructor(options?: MonetaryGrant) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
