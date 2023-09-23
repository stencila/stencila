// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { Node } from './Node';

// A schema specifying that a node must be one of several values.
export class EnumValidator extends Entity {
  type = "EnumValidator";

  // A node is valid if it is equal to any of these values.
  values: Node[];

  constructor(values: Node[], options?: EnumValidator) {
    super()
    if (options) Object.assign(this, options)
    this.values = values;
  }
}
