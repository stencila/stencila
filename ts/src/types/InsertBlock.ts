// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to insert some block content.
 */
export class InsertBlock extends SuggestionBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InsertBlock";

  constructor(content: Block[], options?: Partial<InsertBlock>) {
    super(content);
    this.type = "InsertBlock";
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
