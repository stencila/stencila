// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Hint } from "./Hint.js";
import { Integer } from "./Integer.js";

/**
 * A hint to the structure of an `Object`.
 */
export class ObjectHint extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ObjectHint";

  /**
   * The number of entries in the object.
   */
  length: Integer;

  /**
   * The keys of the object's entries.
   */
  keys: string[];

  /**
   * Hints to the values of the object's entries.
   */
  values: Hint[];

  constructor(length: Integer, keys: string[], values: Hint[], options?: Partial<ObjectHint>) {
    super();
    this.type = "ObjectHint";
    if (options) Object.assign(this, options);
    this.length = length;
    this.keys = keys;
    this.values = values;
  }
}

/**
* Create a new `ObjectHint`
*/
export function objectHint(length: Integer, keys: string[], values: Hint[], options?: Partial<ObjectHint>): ObjectHint {
  return new ObjectHint(length, keys, values, options);
}
