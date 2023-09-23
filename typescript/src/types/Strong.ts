// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Mark } from './Mark';

// Strongly emphasized content.
export class Strong extends Mark {
  type = "Strong";

  constructor(content: Inline[], options?: Strong) {
    super(content)
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
