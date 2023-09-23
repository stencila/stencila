// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { Node } from './Node';

// A validator specifying a constant value that a node must have.
export class ConstantValidator extends Entity {
  type = "ConstantValidator";

  // The value that the node must have.
  value: Node;

  constructor(value: Node, options?: ConstantValidator) {
    super()
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
