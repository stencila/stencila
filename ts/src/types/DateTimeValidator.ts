// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { DateTime } from "./DateTime.js";
import { Entity } from "./Entity.js";

/**
 * A validator specifying the constraints on a date-time.
 */
export class DateTimeValidator extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DateTimeValidator";

  /**
   * The inclusive lower limit for a date-time.
   */
  minimum?: DateTime;

  /**
   * The inclusive upper limit for a date-time.
   */
  maximum?: DateTime;

  constructor(options?: Partial<DateTimeValidator>) {
    super();
    this.type = "DateTimeValidator";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `DateTimeValidator`
*/
export function dateTimeValidator(options?: Partial<DateTimeValidator>): DateTimeValidator {
  return new DateTimeValidator(options);
}
