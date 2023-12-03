// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * Abstract base type for nodes that indicate a suggested change to content.
 */
export class Suggestion extends Entity {
  type = "Suggestion";

  constructor(options?: Partial<Suggestion>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Suggestion`
*/
export function suggestion(options?: Partial<Suggestion>): Suggestion {
  return new Suggestion(options);
}
