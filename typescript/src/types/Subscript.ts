// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Mark } from './Mark';

// Subscripted content.
export class Subscript extends Mark {
  type = "Subscript";

  constructor(content: Inline[], options?: Subscript) {
    super(content)
    if (options) Object.assign(this, options)
    this.content = content;
  }

  static from(other: Subscript): Subscript {
    return new Subscript(other.content!, other)
  }
}
