// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Suggestion } from "./Suggestion.js";

/**
 * Abstract base type for nodes that indicate a suggested change to inline content.
 */
export class SuggestionInline extends Suggestion {
  type = "SuggestionInline";

  /**
   * The content that is suggested to be inserted, modified, replaced, or deleted.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<SuggestionInline>) {
    super();
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
