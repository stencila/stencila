// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Subscripted content.
export class Subscript {
  // The type of this item
  type = "Subscript";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Subscript) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
