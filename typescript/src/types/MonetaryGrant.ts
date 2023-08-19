// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PersonOrOrganization } from './PersonOrOrganization';
import { PropertyValueOrString } from './PropertyValueOrString';
import { Thing } from './Thing';

// A monetary grant.
export class MonetaryGrant {
  type = "MonetaryGrant";

  // The identifier for this item
  id?: string;

  // Alternate names (aliases) for the item.
  alternateNames?: string[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: string;

  // The URL of the item.
  url?: string;

  // Indicates an item funded or sponsored through a Grant.
  fundedItems?: Thing[];

  // A person or organization that supports a thing through a pledge, promise, or financial contribution.
  sponsors?: PersonOrOrganization[];

  // The amount of money.
  amounts?: number;

  // A person or organization that supports (sponsors) something through some kind of financial contribution.
  funders?: PersonOrOrganization[];

  constructor(options?: MonetaryGrant) {
    if (options) Object.assign(this, options)
    
  }
}
