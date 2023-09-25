// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

// Strongly emphasized content.
export class Strong extends Mark {
  type = "Strong";

  constructor(content: Inline[], options?: Strong) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }

  static from(other: Strong): Strong {
    return new Strong(other.content!, other);
  }
}
