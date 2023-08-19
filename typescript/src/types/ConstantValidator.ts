// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Node } from './Node';
import { String } from './String';

// A validator specifying a constant value that a node must have.
export class ConstantValidator {
  // The type of this item
  type = "ConstantValidator";

  // The identifier for this item
  id?: String;

  // The value that the node must have.
  value: Node;

  constructor(value: Node, options?: ConstantValidator) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
