// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Duration } from './Duration';
import { TimeUnit } from './TimeUnit';

// A validator specifying the constraints on a duration.
export class DurationValidator {
  type = "DurationValidator";

  // The identifier for this item
  id?: string;

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
