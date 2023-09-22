// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';

// A suggestion to delete some inline content.
export class Delete {
  type = "Delete";

  // The identifier for this item
  id?: string;

  // The content that is suggested to be inserted or deleted.
  content: Inline[];

  constructor(content: Inline[], options?: Delete) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
