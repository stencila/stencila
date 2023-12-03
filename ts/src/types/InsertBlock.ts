// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to insert some block content.
 */
export class InsertBlock extends SuggestionBlock {
  type = "InsertBlock";

  constructor(content: Block[], options?: Partial<InsertBlock>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `InsertBlock`
*/
export function insertBlock(content: Block[], options?: Partial<InsertBlock>): InsertBlock {
  return new InsertBlock(content, options);
}
