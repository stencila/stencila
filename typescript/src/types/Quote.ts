// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CiteOrString } from './CiteOrString';
import { Inline } from './Inline';
import { String } from './String';

// Inline, quoted content.
export class Quote {
  // The type of this item
  type = "Quote";

  // The identifier for this item
  id?: String;

  // The content that is marked.
  content: Inline[];

  // The source of the quote.
  cite?: CiteOrString;

  constructor(content: Inline[], options?: Quote) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
