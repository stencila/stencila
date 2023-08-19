// Generated file; do not edit. See `../rust/schema-gen` crate.

import { TimeUnit } from './TimeUnit';
import { Timestamp } from './Timestamp';

// A validator specifying the constraints on a timestamp.
export class TimestampValidator {
  type = "TimestampValidator";

  // The identifier for this item
  id?: string;

  // The time units that the timestamp can have.
  timeUnits?: TimeUnit[];

  // The inclusive lower limit for a timestamp.
  minimum?: Timestamp;

  // The inclusive upper limit for a timestamp.
  maximum?: Timestamp;

  constructor(options?: TimestampValidator) {
    if (options) Object.assign(this, options)
    
  }
}
