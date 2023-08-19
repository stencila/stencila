// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Subscripted content.
export class Subscript {
  type = "Subscript";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Subscript) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
