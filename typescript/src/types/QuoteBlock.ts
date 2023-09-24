// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CiteOrString } from './CiteOrString';
import { Entity } from './Entity';

// A section quoted from somewhere else.
export class QuoteBlock extends Entity {
  type = "QuoteBlock";

  // The source of the quote.
  cite?: CiteOrString;

  // The content of the quote.
  content: Block[];

  constructor(content: Block[], options?: QuoteBlock) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
  }

  static from(other: QuoteBlock): QuoteBlock {
    return new QuoteBlock(other.content!, other)
  }
}
