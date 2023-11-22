// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to delete some block content.
 */
export class DeleteBlock extends SuggestionBlock {
  type = "DeleteBlock";

  constructor(content: Block[], options?: Partial<DeleteBlock>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `DeleteBlock`
*/
export function deleteBlock(content: Block[], options?: Partial<DeleteBlock>): DeleteBlock {
  return new DeleteBlock(content, options);
}
