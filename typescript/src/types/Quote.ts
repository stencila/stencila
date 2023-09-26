// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CiteOrString } from "./CiteOrString.js";
import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Inline, quoted content.
 */
export class Quote extends Mark {
  type = "Quote";

  /**
   * The source of the quote.
   */
  cite?: CiteOrString;

  constructor(content: Inline[], options?: Partial<Quote>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Quote`
*/
export function quote(content: Inline[], options?: Partial<Quote>): Quote {
  return new Quote(content, options);
}
