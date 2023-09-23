// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { Inline } from './Inline';

// Abstract base class for nodes that mark some other inline content
  // in some way (e.g. as being emphasised, or quoted).
export class Mark extends Entity {
  type = "Mark";

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Mark) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
