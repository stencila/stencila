// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Date } from './Date';
import { Entity } from './Entity';

// A validator specifying the constraints on a date.
export class DateValidator extends Entity {
  type = "DateValidator";

  // The inclusive lower limit for a date.
  minimum?: Date;

  // The inclusive upper limit for a date.
  maximum?: Date;

  constructor(options?: DateValidator) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
