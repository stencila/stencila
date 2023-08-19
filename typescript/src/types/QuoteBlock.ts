// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CiteOrString } from './CiteOrString';

// A section quoted from somewhere else.
export class QuoteBlock {
  type = "QuoteBlock";

  // The identifier for this item
  id?: string;

  // The source of the quote.
  cite?: CiteOrString;

  // The content of the quote.
  content: Block[];

  constructor(content: Block[], options?: QuoteBlock) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
