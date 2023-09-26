// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Validator } from "./Validator.js";

/**
 * A validator specifying constraints on an array of heterogeneous items.
 */
export class TupleValidator extends Entity {
  type = "TupleValidator";

  /**
   * An array of validators specifying the constraints on each successive item in the array.
   */
  items?: Validator[];

  constructor(options?: Partial<TupleValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `TupleValidator`
*/
export function tupleValidator(options?: Partial<TupleValidator>): TupleValidator {
  return new TupleValidator(options);
}
