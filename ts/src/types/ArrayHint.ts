// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { Primitive } from "./Primitive.js";

/**
 * A hint to the content of an `Array`.
 */
export class ArrayHint extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ArrayHint";

  /**
   * The length (number of items) of the array.
   */
  length: Integer;

  /**
   * The distinct types of the array items.
   */
  itemTypes?: string[];

  /**
   * The minimum value in the array.
   */
  minimum?: Primitive;

  /**
   * The maximum value in the array.
   */
  maximum?: Primitive;

  /**
   * The number of `Null` values in the array.
   */
  nulls?: Integer;

  constructor(length: Integer, options?: Partial<ArrayHint>) {
    super();
    this.type = "ArrayHint";
    if (options) Object.assign(this, options);
    this.length = length;
  }
}

/**
* Create a new `ArrayHint`
*/
export function arrayHint(length: Integer, options?: Partial<ArrayHint>): ArrayHint {
  return new ArrayHint(length, options);
}
