// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * Abstract base class for nodes that indicate a suggested change to inline content.
 */
export class Suggestion extends Entity {
  type = "Suggestion";

  /**
   * The content that is suggested to be inserted or deleted.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<Suggestion>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Suggestion`
*/
export function suggestion(content: Inline[], options?: Partial<Suggestion>): Suggestion {
  return new Suggestion(content, options);
}
