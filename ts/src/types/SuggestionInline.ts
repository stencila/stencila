// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Suggestion } from "./Suggestion.js";

/**
 * Abstract base type for nodes that indicate a suggested change to inline content.
 */
export class SuggestionInline extends Suggestion {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "SuggestionInline";

  /**
   * The content that is suggested to be inserted, modified, replaced, or deleted.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<SuggestionInline>) {
    super();
    this.type = "SuggestionInline";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `SuggestionInline`
*/
export function suggestionInline(content: Inline[], options?: Partial<SuggestionInline>): SuggestionInline {
  return new SuggestionInline(content, options);
}
