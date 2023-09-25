// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Subscripted content.
 */
export class Subscript extends Mark {
  type = "Subscript";

  constructor(content: Inline[], options?: Partial<Subscript>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `Subscript` from an object
  */
  static from(other: Subscript): Subscript {
    return new Subscript(other.content!, other);
  }
}
