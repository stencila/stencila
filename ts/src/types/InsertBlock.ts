// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to insert some block content.
 */
export class InsertBlock extends SuggestionBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InsertBlock";

  constructor(suggestionStatus: SuggestionStatus, content: Block[], options?: Partial<InsertBlock>) {
    super(suggestionStatus, content);
    this.type = "InsertBlock";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
  }
}

/**
* Create a new `InsertBlock`
*/
export function insertBlock(suggestionStatus: SuggestionStatus, content: Block[], options?: Partial<InsertBlock>): InsertBlock {
  return new InsertBlock(suggestionStatus, content, options);
}
