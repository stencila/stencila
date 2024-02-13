// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A type to indicate a value or or other type in unknown.
 */
export class Unknown extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Unknown";

  constructor(options?: Partial<Unknown>) {
    super();
    this.type = "Unknown";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Unknown`
*/
export function unknown(options?: Partial<Unknown>): Unknown {
  return new Unknown(options);
}
