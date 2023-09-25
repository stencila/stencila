// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

// Emphasized content.
export class Emphasis extends Mark {
  type = "Emphasis";

  constructor(content: Inline[], options?: Emphasis) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  static from(other: Emphasis): Emphasis {
    return new Emphasis(other.content!, other);
  }
}
