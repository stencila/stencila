// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Duration } from './Duration';
import { String } from './String';
import { TimeUnit } from './TimeUnit';

// A validator specifying the constraints on a duration.
export class DurationValidator {
  // The type of this item
  type = "DurationValidator";

  // The identifier for this item
  id?: String;

  // The time units that the duration can have.
  timeUnits?: TimeUnit[];

  // The inclusive lower limit for a duration.
  minimum?: Duration;

  // The inclusive upper limit for a duration.
  maximum?: Duration;

  constructor(options?: DurationValidator) {
    if (options) Object.assign(this, options)
    
  }
}
