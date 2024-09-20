// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to replace some block content with new block content.
 */
export class ReplaceBlock extends SuggestionBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ReplaceBlock";

  /**
   * The new replacement block content.
   */
  replacement: Block[];

  constructor(content: Block[], replacement: Block[], options?: Partial<ReplaceBlock>) {
    super(content);
    this.type = "ReplaceBlock";
    if (options) Object.assign(this, options);
    this.content = content;
    this.replacement = replacement;
  }
}

/**
* Create a new `ReplaceBlock`
*/
export function replaceBlock(content: Block[], replacement: Block[], options?: Partial<ReplaceBlock>): ReplaceBlock {
  return new ReplaceBlock(content, replacement, options);
}
