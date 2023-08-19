// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PersonOrOrganization } from './PersonOrOrganization';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';
import { Thing } from './Thing';

// A grant, typically financial or otherwise quantifiable, of resources.
export class Grant {
  // The type of this item
  type = "Grant";

  // The identifier for this item
  id?: String;

  // Alternate names (aliases) for the item.
  alternateNames?: String[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: String;

  // The URL of the item.
  url?: String;

  // Indicates an item funded or sponsored through a Grant.
  fundedItems?: Thing[];

  // A person or organization that supports a thing through a pledge, promise, or financial contribution.
  sponsors?: PersonOrOrganization[];

  constructor(options?: Grant) {
    if (options) Object.assign(this, options)
    
  }
}
