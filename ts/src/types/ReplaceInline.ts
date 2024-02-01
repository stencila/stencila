// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { SuggestionInline } from "./SuggestionInline.js";

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

  constructor(content: Inline[], replacement: Inline[], options?: Partial<ReplaceInline>) {
    super(content);
    this.type = "ReplaceInline";
    if (options) Object.assign(this, options);
    this.content = content;
    this.replacement = replacement;
  }
}

/**
* Create a new `ReplaceInline`
*/
export function replaceInline(content: Inline[], replacement: Inline[], options?: Partial<ReplaceInline>): ReplaceInline {
  return new ReplaceInline(content, replacement, options);
}
