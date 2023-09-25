// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Emphasized content.
 */
export class Emphasis extends Mark {
  type = "Emphasis";

  constructor(content: Inline[], options?: Partial<Emphasis>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `Emphasis` from an object
  */
  static from(other: Emphasis): Emphasis {
    return new Emphasis(other.content!, other);
  }
}
