// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Node } from './Node';
import { String } from './String';

// A schema specifying that a node must be one of several values.
export class EnumValidator {
  // The type of this item
  type = "EnumValidator";

  // The identifier for this item
  id?: String;

  // A node is valid if it is equal to any of these values.
  values: Node[];

  constructor(values: Node[], options?: EnumValidator) {
    if (options) Object.assign(this, options)
    this.values = values;
  }
}
