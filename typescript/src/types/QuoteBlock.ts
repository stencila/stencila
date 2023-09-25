// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { CiteOrString } from "./CiteOrString.js";
import { Entity } from "./Entity.js";

/**
 * A section quoted from somewhere else.
 */
export class QuoteBlock extends Entity {
  type = "QuoteBlock";

  /**
   * The source of the quote.
   */
  cite?: CiteOrString;

  /**
   * The content of the quote.
   */
  content: Block[];

  constructor(content: Block[], options?: Partial<QuoteBlock>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `QuoteBlock` from an object
  */
  static from(other: QuoteBlock): QuoteBlock {
    return new QuoteBlock(other.content!, other);
  }
}

/**
* Create a new `QuoteBlock`
*/
export function quoteBlock(content: Block[], options?: Partial<QuoteBlock>): QuoteBlock {
  return new QuoteBlock(content, options);
}
