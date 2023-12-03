// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Date } from "./Date.js";
import { Entity } from "./Entity.js";

/**
 * A validator specifying the constraints on a date.
 */
export class DateValidator extends Entity {
  type = "DateValidator";

  /**
   * The inclusive lower limit for a date.
   */
  minimum?: Date;

  /**
   * The inclusive upper limit for a date.
   */
  maximum?: Date;

  constructor(options?: Partial<DateValidator>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `DateValidator`
*/
export function dateValidator(options?: Partial<DateValidator>): DateValidator {
  return new DateValidator(options);
}
