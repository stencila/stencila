// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";

/**
 * A suggestion to insert some inline content.
 */
export class InsertInline extends SuggestionInline {
  type = "InsertInline";

  constructor(content: Inline[], options?: Partial<InsertInline>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `InsertInline`
*/
export function insertInline(content: Inline[], options?: Partial<InsertInline>): InsertInline {
  return new InsertInline(content, options);
}
