// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A schema specifying that a node must be a boolean value.
 */
export class BooleanValidator extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "BooleanValidator";

  constructor(options?: Partial<BooleanValidator>) {
    super();
    this.type = "BooleanValidator";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `BooleanValidator`
*/
export function booleanValidator(options?: Partial<BooleanValidator>): BooleanValidator {
  return new BooleanValidator(options);
}
