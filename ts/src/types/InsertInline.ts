// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";

/**
 * A suggestion to insert some inline content.
 */
export class InsertInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InsertInline";

  constructor(content: Inline[], options?: Partial<InsertInline>) {
    super(content);
    this.type = "InsertInline";
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
