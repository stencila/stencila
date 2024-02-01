// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Validator } from "./Validator.js";

/**
 * A validator specifying constraints on an array of heterogeneous items.
 */
export class TupleValidator extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "TupleValidator";

  /**
   * An array of validators specifying the constraints on each successive item in the array.
   */
  items?: Validator[];

  constructor(options?: Partial<TupleValidator>) {
    super();
    this.type = "TupleValidator";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `TupleValidator`
*/
export function tupleValidator(options?: Partial<TupleValidator>): TupleValidator {
  return new TupleValidator(options);
}
