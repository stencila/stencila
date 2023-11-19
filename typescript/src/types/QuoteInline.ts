// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CiteOrText } from "./CiteOrText.js";
import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Inline, quoted content.
 */
export class QuoteInline extends Mark {
  type = "QuoteInline";

  /**
   * The source of the quote.
   */
  cite?: CiteOrText;

  constructor(content: Inline[], options?: Partial<QuoteInline>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `QuoteInline`
*/
export function quoteInline(content: Inline[], options?: Partial<QuoteInline>): QuoteInline {
  return new QuoteInline(content, options);
}
