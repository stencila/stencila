// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// Paragraph
export class Paragraph {
  type = "Paragraph";

  // The identifier for this item
  id?: string;

  // The contents of the paragraph.
  content: Inline[];

  constructor(content: Inline[], options?: Paragraph) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
