// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// Lists or enumerations, for example, a list of cuisines or music genres, etc.
export class Enumeration {
  type = "Enumeration";

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

  constructor(options?: Enumeration) {
    if (options) Object.assign(this, options)
    
  }
}
