// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Emphasized content.
export class Emphasis {
  type = "Emphasis";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Emphasis) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
