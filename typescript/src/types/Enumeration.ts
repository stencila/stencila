// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Thing } from './Thing';

// Lists or enumerations, for example, a list of cuisines or music genres, etc.
export class Enumeration extends Thing {
  type = "Enumeration";

  constructor(options?: Enumeration) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: Enumeration): Enumeration {
    return new Enumeration(other)
  }
}
