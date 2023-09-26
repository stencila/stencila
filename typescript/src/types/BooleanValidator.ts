// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * A schema specifying that a node must be a boolean value.
 */
export class BooleanValidator extends Entity {
  type = "BooleanValidator";

  constructor(options?: Partial<BooleanValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `BooleanValidator`
*/
export function booleanValidator(options?: Partial<BooleanValidator>): BooleanValidator {
  return new BooleanValidator(options);
}
