// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";
import { ImageObject } from "./ImageObject.js";
import { PropertyValueOrString } from "./PropertyValueOrString.js";

/**
 * The most generic type of item.
 */
export class Thing extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Thing";

  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];

  /**
   * A description of the item.
   */
  description?: Cord;

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
    this.type = "Thing";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Thing`
*/
export function thing(options?: Partial<Thing>): Thing {
  return new Thing(options);
}
