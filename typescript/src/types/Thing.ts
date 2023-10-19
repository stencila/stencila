// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { ImageObject } from "./ImageObject.js";
import { PropertyValueOrString } from "./PropertyValueOrString.js";
import { Text } from "./Text.js";

/**
 * The most generic type of item.
 */
export class Thing extends Entity {
  type = "Thing";

  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];

  /**
   * A description of the item.
   */
  description?: Text;

  /**
   * Any kind of identifier for any kind of Thing.
   */
  identifiers?: PropertyValueOrString[];

  /**
   * Images of the item.
   */
  images?: ImageObject[];

  /**
   * The name of the item.
   */
  name?: string;

  /**
   * The URL of the item.
   */
  url?: string;

  constructor(options?: Partial<Thing>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Thing`
*/
export function thing(options?: Partial<Thing>): Thing {
  return new Thing(options);
}
