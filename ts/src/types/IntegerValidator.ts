// Generated file; do not edit. See `../rust/schema-gen` crate.

import { NumberValidator } from "./NumberValidator.js";

/**
 * A validator specifying the constraints on an integer node.
 */
export class IntegerValidator extends NumberValidator {
  type = "IntegerValidator";

  constructor(options?: Partial<IntegerValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `IntegerValidator`
*/
export function integerValidator(options?: Partial<IntegerValidator>): IntegerValidator {
  return new IntegerValidator(options);
}
