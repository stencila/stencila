// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { ImageObjectOrString } from "./ImageObjectOrString.js";
import { PropertyValueOrString } from "./PropertyValueOrString.js";

// The most generic type of item.
export class Thing extends Entity {
  type = "Thing";

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

  constructor(options?: Thing) {
    super();
    if (options) Object.assign(this, options);
    
  }

  static from(other: Thing): Thing {
    return new Thing(other);
  }
}
