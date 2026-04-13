// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Suggestion } from "./Suggestion.js";

/**
 * Abstract base type for nodes that indicate a suggested change to block content.
 */
export class SuggestionBlock extends Suggestion {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "SuggestionBlock";

  /**
   * The suggested content. For insertions and replacements, this is the new content; for deletions, this is the content being deleted.
   */
  content: Block[];

  /**
   * The original content. For replacements, this is the content being replaced; for deletions, this should be absent.
   */
  original?: Block[];

  constructor(content: Block[], options?: Partial<SuggestionBlock>) {
    super();
    this.type = "SuggestionBlock";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `SuggestionBlock`
*/
export function suggestionBlock(content: Block[], options?: Partial<SuggestionBlock>): SuggestionBlock {
  return new SuggestionBlock(content, options);
}
