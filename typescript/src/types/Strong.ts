// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Strongly emphasized content.
export class Strong {
  type = "Strong";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Strong) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
