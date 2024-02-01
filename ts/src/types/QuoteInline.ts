// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CiteOrText } from "./CiteOrText.js";
import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Inline, quoted content.
 */
export class QuoteInline extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "QuoteInline";

  /**
   * The source of the quote.
   */
  cite?: CiteOrText;

  constructor(content: Inline[], options?: Partial<QuoteInline>) {
    super(content);
    this.type = "QuoteInline";
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
