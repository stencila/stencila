// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Inline text that is underlined.
export class Underline {
  // The type of this item
  type = "Underline";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Underline) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
