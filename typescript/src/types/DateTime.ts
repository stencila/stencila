// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
 */
export class DateTime extends Entity {
  type = "DateTime";

  /**
   * The date as an ISO 8601 string.
   */
  value: string;

  constructor(value: string, options?: Partial<DateTime>) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
  }

  /**
  * Create a `DateTime` from an object
  */
  static from(other: DateTime): DateTime {
    return new DateTime(other.value!, other);
  }
}

/**
* Create a new `DateTime`
*/
export function dateTime(value: string, options?: Partial<DateTime>): DateTime {
  return new DateTime(value, options);
}
