// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Duration } from "./Duration.js";
import { Entity } from "./Entity.js";
import { TimeUnit } from "./TimeUnit.js";

/**
 * A validator specifying the constraints on a duration.
 */
export class DurationValidator extends Entity {
  type = "DurationValidator";

  /**
   * The time units that the duration can have.
   */
  timeUnits?: TimeUnit[];

  /**
   * The inclusive lower limit for a duration.
   */
  minimum?: Duration;

  /**
   * The inclusive upper limit for a duration.
   */
  maximum?: Duration;

  constructor(options?: Partial<DurationValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `DurationValidator`
*/
export function durationValidator(options?: Partial<DurationValidator>): DurationValidator {
  return new DurationValidator(options);
}
