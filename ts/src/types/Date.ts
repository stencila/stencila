// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * A calendar date encoded as a ISO 8601 string.
 */
export class Date extends Entity {
  type = "Date";

  /**
   * The date as an ISO 8601 string.
   */
  value: string;

  constructor(value: string, options?: Partial<Date>) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `Date`
*/
export function date(value: string, options?: Partial<Date>): Date {
  return new Date(value, options);
}
