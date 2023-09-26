// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { TimeUnit } from "./TimeUnit.js";
import { Timestamp } from "./Timestamp.js";

/**
 * A validator specifying the constraints on a timestamp.
 */
export class TimestampValidator extends Entity {
  type = "TimestampValidator";

  /**
   * The time units that the timestamp can have.
   */
  timeUnits?: TimeUnit[];

  /**
   * The inclusive lower limit for a timestamp.
   */
  minimum?: Timestamp;

  /**
   * The inclusive upper limit for a timestamp.
   */
  maximum?: Timestamp;

  constructor(options?: Partial<TimestampValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `TimestampValidator`
*/
export function timestampValidator(options?: Partial<TimestampValidator>): TimestampValidator {
  return new TimestampValidator(options);
}
