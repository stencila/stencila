// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';
import { Validator } from './Validator';

// A validator specifying constraints on an array of heterogeneous items.
export class TupleValidator {
  // The type of this item
  type = "TupleValidator";

  // The identifier for this item
  id?: String;

  // An array of validators specifying the constraints on each successive item in the array.
  items?: Validator[];

  constructor(options?: TupleValidator) {
    if (options) Object.assign(this, options)
    
  }
}
