// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

/**
 * A schema specifying that a node must be one of several values.
 */
export class EnumValidator extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "EnumValidator";

  /**
   * A node is valid if it is equal to any of these values.
   */
  values: Node[];

  constructor(values: Node[], options?: Partial<EnumValidator>) {
    super();
    this.type = "EnumValidator";
    if (options) Object.assign(this, options);
    this.values = values;
  }
}

/**
* Create a new `EnumValidator`
*/
export function enumValidator(values: Node[], options?: Partial<EnumValidator>): EnumValidator {
  return new EnumValidator(values, options);
}
