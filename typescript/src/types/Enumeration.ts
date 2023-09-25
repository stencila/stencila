// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Thing } from "./Thing.js";

/**
 * Lists or enumerations, for example, a list of cuisines or music genres, etc.
 */
export class Enumeration extends Thing {
  type = "Enumeration";

  constructor(options?: Partial<Enumeration>) {
    super();
    if (options) Object.assign(this, options);
    
  }

  /**
  * Create a `Enumeration` from an object
  */
  static from(other: Enumeration): Enumeration {
    return new Enumeration(other);
  }
}

/**
* Create a new `Enumeration`
*/
export function enumeration(options?: Partial<Enumeration>): Enumeration {
  return new Enumeration(options);
}
