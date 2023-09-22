// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// A suggestion to insert some inline content.
export class Insert {
  type = "Insert";

  // The identifier for this item
  id?: string;

  // The content that is suggested to be inserted or deleted.
  content: Inline[];

  constructor(content: Inline[], options?: Insert) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
