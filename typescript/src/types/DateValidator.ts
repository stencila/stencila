// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Date } from './Date';

// A validator specifying the constraints on a date.
export class DateValidator {
  type = "DateValidator";

  // The identifier for this item
  id?: string;

  // The inclusive lower limit for a date.
  minimum?: Date;

  // The inclusive upper limit for a date.
  maximum?: Date;

  constructor(options?: DateValidator) {
    if (options) Object.assign(this, options)
    
  }
}
