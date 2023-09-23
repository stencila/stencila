// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from './Inline';
import { Suggestion } from './Suggestion';

// A suggestion to delete some inline content.
export class Delete extends Suggestion {
  type = "Delete";

  constructor(content: Inline[], options?: Delete) {
    super(content)
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
