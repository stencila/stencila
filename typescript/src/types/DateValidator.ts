// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Date } from './Date';
import { String } from './String';

// A validator specifying the constraints on a date.
export class DateValidator {
  // The type of this item
  type = "DateValidator";

  // The identifier for this item
  id?: String;

  // The inclusive lower limit for a date.
  minimum?: Date;

  // The inclusive upper limit for a date.
  maximum?: Date;

  constructor(options?: DateValidator) {
    if (options) Object.assign(this, options)
    
  }
}
