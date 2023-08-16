// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Strongly emphasised content.
export class Strong {
  // The type of this item
  type = "Strong";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Strong) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
