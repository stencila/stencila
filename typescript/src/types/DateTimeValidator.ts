// Generated file; do not edit. See `../rust/schema-gen` crate.

import { DateTime } from './DateTime';

// A validator specifying the constraints on a date-time.
export class DateTimeValidator {
  type = "DateTimeValidator";

  // The identifier for this item
  id?: string;

  // The inclusive lower limit for a date-time.
  minimum?: DateTime;

  // The inclusive upper limit for a date-time.
  maximum?: DateTime;

  constructor(options?: DateTimeValidator) {
    if (options) Object.assign(this, options)
    
  }
}
