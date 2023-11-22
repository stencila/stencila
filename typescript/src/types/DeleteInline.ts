// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";

/**
 * A suggestion to delete some inline content.
 */
export class DeleteInline extends SuggestionInline {
  type = "DeleteInline";

  constructor(content: Inline[], options?: Partial<DeleteInline>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `DeleteInline`
*/
export function deleteInline(content: Inline[], options?: Partial<DeleteInline>): DeleteInline {
  return new DeleteInline(content, options);
}
