// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Content that is marked as struck out
export class Strikeout {
  type = "Strikeout";

  // The identifier for this item
  id?: string;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Strikeout) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
