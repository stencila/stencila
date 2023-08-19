// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Validator } from './Validator';

// A validator specifying constraints on an array of heterogeneous items.
export class TupleValidator {
  type = "TupleValidator";

  // The identifier for this item
  id?: string;

  // An array of validators specifying the constraints on each successive item in the array.
  items?: Validator[];

  constructor(options?: TupleValidator) {
    if (options) Object.assign(this, options)
    
  }
}
