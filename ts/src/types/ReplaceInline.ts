// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to replace some inline content with new inline content.
 */
export class ReplaceInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ReplaceInline";

  /**
   * The new replacement inline content.
   */
  replacement: Inline[];

  constructor(suggestionStatus: SuggestionStatus, content: Inline[], replacement: Inline[], options?: Partial<ReplaceInline>) {
    super(suggestionStatus, content);
    this.type = "ReplaceInline";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
    this.replacement = replacement;
  }
}

/**
* Create a new `ReplaceInline`
*/
export function replaceInline(suggestionStatus: SuggestionStatus, content: Inline[], replacement: Inline[], options?: Partial<ReplaceInline>): ReplaceInline {
  return new ReplaceInline(suggestionStatus, content, replacement, options);
}
