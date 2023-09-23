// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';

// A schema specifying that a node must be a boolean value.
export class BooleanValidator extends Entity {
  type = "BooleanValidator";

  constructor(options?: BooleanValidator) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
