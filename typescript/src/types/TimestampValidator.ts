// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';
import { TimeUnit } from './TimeUnit';
import { Timestamp } from './Timestamp';

// A validator specifying the constraints on a timestamp.
export class TimestampValidator {
  // The type of this item
  type = "TimestampValidator";

  // The identifier for this item
  id?: String;

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
