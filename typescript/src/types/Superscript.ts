// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Mark } from './Mark';

// Superscripted content.
export class Superscript extends Mark {
  type = "Superscript";

  constructor(content: Inline[], options?: Superscript) {
    super(content)
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
