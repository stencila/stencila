// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { Integer } from './Integer';
import { String } from './String';

// A heading.
export class Heading {
  // The type of this item
  type = "Heading";

  // The identifier for this item
  id?: String;

  // The depth of the heading.
  depth: Integer = 1;

  // Content of the heading.
  content: Inline[];

  constructor(depth: Integer, content: Inline[], options?: Heading) {
    if (options) Object.assign(this, options)
    this.depth = depth;
    this.content = content;
  }
}
