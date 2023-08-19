// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CiteOrString } from './CiteOrString';
import { String } from './String';

// A section quoted from somewhere else.
export class QuoteBlock {
  // The type of this item
  type = "QuoteBlock";

  // The identifier for this item
  id?: String;

  // The source of the quote.
  cite?: CiteOrString;

  // The content of the quote.
  content: Block[];

  constructor(content: Block[], options?: QuoteBlock) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
