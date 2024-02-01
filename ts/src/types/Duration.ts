// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { TimeUnit } from "./TimeUnit.js";

/**
 * A value that represents the difference between two timestamps.
 */
export class Duration extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Duration";

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
    this.type = "Duration";
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
