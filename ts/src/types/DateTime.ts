// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
 */
export class DateTime extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DateTime";

  /**
   * The date as an ISO 8601 string.
   */
  value: string;

  constructor(value: string, options?: Partial<DateTime>) {
    super();
    this.type = "DateTime";
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `DateTime`
*/
export function dateTime(value: string, options?: Partial<DateTime>): DateTime {
  return new DateTime(value, options);
}
