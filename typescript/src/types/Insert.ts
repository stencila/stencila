// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Suggestion } from "./Suggestion.js";

/**
 * A suggestion to insert some inline content.
 */
export class Insert extends Suggestion {
  type = "Insert";

  constructor(content: Inline[], options?: Partial<Insert>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `Insert` from an object
  */
  static from(other: Insert): Insert {
    return new Insert(other.content!, other);
  }
}

/**
* Create a new `Insert`
*/
export function insert(content: Inline[], options?: Partial<Insert>): Insert {
  return new Insert(content, options);
}
