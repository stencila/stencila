// Generated file; do not edit. See `../rust/schema-gen` crate.

import { DateTime } from './DateTime';
import { Entity } from './Entity';

// A validator specifying the constraints on a date-time.
export class DateTimeValidator extends Entity {
  type = "DateTimeValidator";

  // The inclusive lower limit for a date-time.
  minimum?: DateTime;

  // The inclusive upper limit for a date-time.
  maximum?: DateTime;

  constructor(options?: DateTimeValidator) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: DateTimeValidator): DateTimeValidator {
    return new DateTimeValidator(other)
  }
}
