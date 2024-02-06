// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Time } from "./Time.js";

/**
 * A validator specifying the constraints on a time.
 */
export class TimeValidator extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "TimeValidator";

  /**
   * The inclusive lower limit for a time.
   */
  minimum?: Time;

  /**
   * The inclusive upper limit for a time.
   */
  maximum?: Time;

  constructor(options?: Partial<TimeValidator>) {
    super();
    this.type = "TimeValidator";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `TimeValidator`
*/
export function timeValidator(options?: Partial<TimeValidator>): TimeValidator {
  return new TimeValidator(options);
}
