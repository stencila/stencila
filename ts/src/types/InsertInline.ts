// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to insert some inline content.
 */
export class InsertInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InsertInline";

  constructor(suggestionStatus: SuggestionStatus, content: Inline[], options?: Partial<InsertInline>) {
    super(suggestionStatus, content);
    this.type = "InsertInline";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
  }
}

/**
* Create a new `InsertInline`
*/
export function insertInline(suggestionStatus: SuggestionStatus, content: Inline[], options?: Partial<InsertInline>): InsertInline {
  return new InsertInline(suggestionStatus, content, options);
}
