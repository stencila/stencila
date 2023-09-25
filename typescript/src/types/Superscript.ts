// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Superscripted content.
 */
export class Superscript extends Mark {
  type = "Superscript";

  constructor(content: Inline[], options?: Partial<Superscript>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `Superscript` from an object
  */
  static from(other: Superscript): Superscript {
    return new Superscript(other.content!, other);
  }
}
