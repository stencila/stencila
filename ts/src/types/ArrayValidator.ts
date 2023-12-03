// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { Validator } from "./Validator.js";

/**
 * A validator specifying constraints on an array node.
 */
export class ArrayValidator extends Entity {
  type = "ArrayValidator";

  /**
   * Whether items can have the value `Node::Null`
   */
  itemsNullable?: boolean;

  /**
   * Another validator node specifying the constraints on all items in the array.
   */
  itemsValidator?: Validator;

  /**
   * An array node is valid if at least one of its items is valid against the `contains` schema.
   */
  contains?: Validator;

  /**
   * An array node is valid if its size is greater than, or equal to, this value.
   */
  minItems?: Integer;

  /**
   * An array node is valid if its size is less than, or equal to, this value.
   */
  maxItems?: Integer;

  /**
   * A flag to indicate that each value in the array should be unique.
   */
  uniqueItems?: boolean;

  constructor(options?: Partial<ArrayValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ArrayValidator`
*/
export function arrayValidator(options?: Partial<ArrayValidator>): ArrayValidator {
  return new ArrayValidator(options);
}
