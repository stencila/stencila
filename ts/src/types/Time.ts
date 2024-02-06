// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A point in time recurring on multiple days.
 */
export class Time extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Time";

  /**
   * The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
   */
  value: string;

  constructor(value: string, options?: Partial<Time>) {
    super();
    this.type = "Time";
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `Time`
*/
export function time(value: string, options?: Partial<Time>): Time {
  return new Time(value, options);
}
