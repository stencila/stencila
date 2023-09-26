// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

/**
 * A schema specifying that a node must be one of several values.
 */
export class EnumValidator extends Entity {
  type = "EnumValidator";

  /**
   * A node is valid if it is equal to any of these values.
   */
  values: Node[];

  constructor(values: Node[], options?: Partial<EnumValidator>) {
    super();
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
