// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Superscripted content.
export class Superscript {
  type = "Superscript";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Superscript) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
