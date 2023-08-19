// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Inline text that is underlined.
export class Underline {
  type = "Underline";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Underline) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
