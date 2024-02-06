// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A calendar date encoded as a ISO 8601 string.
 */
export class Date extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Date";

  /**
   * The date as an ISO 8601 string.
   */
  value: string;

  constructor(value: string, options?: Partial<Date>) {
    super();
    this.type = "Date";
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
