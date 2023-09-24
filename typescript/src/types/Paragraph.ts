// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { Inline } from './Inline';

// Paragraph
export class Paragraph extends Entity {
  type = "Paragraph";

  // The contents of the paragraph.
  content: Inline[];

  constructor(content: Inline[], options?: Paragraph) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
  }

  static from(other: Paragraph): Paragraph {
    return new Paragraph(other.content!, other)
  }
}
