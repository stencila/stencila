// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { Integer } from "./Integer.js";

// A heading.
export class Heading extends Entity {
  type = "Heading";

  // The depth of the heading.
  depth: Integer = 1;

  // Content of the heading.
  content: Inline[];

  constructor(depth: Integer, content: Inline[], options?: Heading) {
    super();
    if (options) Object.assign(this, options);
    this.depth = depth;
    this.content = content;
  }

  static from(other: Heading): Heading {
    return new Heading(other.depth!, other.content!, other);
  }
}
