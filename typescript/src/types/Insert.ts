// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Suggestion } from './Suggestion';

// A suggestion to insert some inline content.
export class Insert extends Suggestion {
  type = "Insert";

  constructor(content: Inline[], options?: Insert) {
    super(content)
    if (options) Object.assign(this, options)
    this.content = content;
  }

  static from(other: Insert): Insert {
    return new Insert(other.content!, other)
  }
}
