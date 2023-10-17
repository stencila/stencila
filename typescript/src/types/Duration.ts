// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { TimeUnit } from "./TimeUnit.js";

/**
 * A value that represents the difference between two timestamps.
 */
export class Duration extends Entity {
  type = "Duration";

  /**
   * The time difference in `timeUnit`s.
   */
  value: Integer;

  /**
   * The time unit that the `value` represents.
   */
  timeUnit: TimeUnit;

  constructor(value: Integer, timeUnit: TimeUnit, options?: Partial<Duration>) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
    this.timeUnit = timeUnit;
  }
}

/**
* Create a new `Duration`
*/
export function duration(value: Integer, timeUnit: TimeUnit, options?: Partial<Duration>): Duration {
  return new Duration(value, timeUnit, options);
}
