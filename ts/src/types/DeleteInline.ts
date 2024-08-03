// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to delete some inline content.
 */
export class DeleteInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DeleteInline";

  constructor(suggestionStatus: SuggestionStatus, content: Inline[], options?: Partial<DeleteInline>) {
    super(suggestionStatus, content);
    this.type = "DeleteInline";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
  }
}

/**
* Create a new `DeleteInline`
*/
export function deleteInline(suggestionStatus: SuggestionStatus, content: Inline[], options?: Partial<DeleteInline>): DeleteInline {
  return new DeleteInline(suggestionStatus, content, options);
}
