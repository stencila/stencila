// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to delete some block content.
 */
export class DeleteBlock extends SuggestionBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DeleteBlock";

  constructor(content: Block[], options?: Partial<DeleteBlock>) {
    super(content);
    this.type = "DeleteBlock";
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
