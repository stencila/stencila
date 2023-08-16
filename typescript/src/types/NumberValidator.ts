// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Number } from './Number';
import { String } from './String';

// A validator specifying the constraints on a numeric node.
export class NumberValidator {
  // The type of this item
  type = "NumberValidator";

  // The identifier for this item
  id?: String;

  // The inclusive lower limit for a numeric node.
  minimum?: Number;

  // The exclusive lower limit for a numeric node.
  exclusiveMinimum?: Number;

  // The inclusive upper limit for a numeric node.
  maximum?: Number;

  // The exclusive upper limit for a numeric node.
  exclusiveMaximum?: Number;

  // A number that a numeric node must be a multiple of.
  multipleOf?: Number;

  constructor(options?: NumberValidator) {
    if (options) Object.assign(this, options)
    
  }
}
