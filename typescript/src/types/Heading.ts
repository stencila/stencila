// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Integer } from './Integer';

// A heading.
export class Heading {
  type = "Heading";

  // The identifier for this item
  id?: string;

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
