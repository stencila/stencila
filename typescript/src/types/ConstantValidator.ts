// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Node } from './Node';

// A validator specifying a constant value that a node must have.
export class ConstantValidator {
  type = "ConstantValidator";

  // The identifier for this item
  id?: string;

  // The value that the node must have.
  value: Node;

  constructor(value: Node, options?: ConstantValidator) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
