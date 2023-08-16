// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Inline } from './Inline';
import { String } from './String';

// Emphasized content.
export class Emphasis {
  // The type of this item
  type = "Emphasis";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  constructor(content: Inline[], options?: Emphasis) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
