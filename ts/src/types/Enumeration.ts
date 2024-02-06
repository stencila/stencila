// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Thing } from "./Thing.js";

/**
 * Lists or enumerations, for example, a list of cuisines or music genres, etc.
 */
export class Enumeration extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Enumeration";

  constructor(options?: Partial<Enumeration>) {
    super();
    this.type = "Enumeration";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Enumeration`
*/
export function enumeration(options?: Partial<Enumeration>): Enumeration {
  return new Enumeration(options);
}
