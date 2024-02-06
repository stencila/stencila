// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { NumberValidator } from "./NumberValidator.js";

/**
 * A validator specifying the constraints on an integer node.
 */
export class IntegerValidator extends NumberValidator {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "IntegerValidator";

  constructor(options?: Partial<IntegerValidator>) {
    super();
    this.type = "IntegerValidator";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `IntegerValidator`
*/
export function integerValidator(options?: Partial<IntegerValidator>): IntegerValidator {
  return new IntegerValidator(options);
}
