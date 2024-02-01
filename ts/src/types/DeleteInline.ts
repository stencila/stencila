// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";

/**
 * A suggestion to delete some inline content.
 */
export class DeleteInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DeleteInline";

  constructor(content: Inline[], options?: Partial<DeleteInline>) {
    super(content);
    this.type = "DeleteInline";
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
