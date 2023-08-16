// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';

// A schema specifying that a node must be a boolean value.
export class BooleanValidator {
  // The type of this item
  type = "BooleanValidator";

  // The identifier for this item
  id?: String;

  constructor(options?: BooleanValidator) {
    if (options) Object.assign(this, options)
    
  }
}
