// Generated file; do not edit. See `../rust/schema-gen` crate.

import { NumberValidator } from './NumberValidator';

// A validator specifying the constraints on an integer node.
export class IntegerValidator extends NumberValidator {
  type = "IntegerValidator";

  constructor(options?: IntegerValidator) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: IntegerValidator): IntegerValidator {
    return new IntegerValidator(other)
  }
}
