// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

/**
 * A validator specifying a constant value that a node must have.
 */
export class ConstantValidator extends Entity {
  type = "ConstantValidator";

  /**
   * The value that the node must have.
   */
  value: Node;

  constructor(value: Node, options?: Partial<ConstantValidator>) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `ConstantValidator`
*/
export function constantValidator(value: Node, options?: Partial<ConstantValidator>): ConstantValidator {
  return new ConstantValidator(value, options);
}
