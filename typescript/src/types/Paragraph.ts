// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Paragraph
export class Paragraph {
  // The type of this item
  type = "Paragraph";

  // The identifier for this item
  id?: String;

  // The contents of the paragraph.
  content: Inline[];

  constructor(content: Inline[], options?: Paragraph) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
