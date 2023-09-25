// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

// Inline text that is underlined.
export class Underline extends Mark {
  type = "Underline";

  constructor(content: Inline[], options?: Underline) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  static from(other: Underline): Underline {
    return new Underline(other.content!, other);
  }
}
