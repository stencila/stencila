// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Content that is marked as struck out
export class Strikeout {
  // The type of this item
  type = "Strikeout";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Strikeout) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
