// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Superscripted content.
export class Superscript {
  // The type of this item
  type = "Superscript";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Superscript) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
