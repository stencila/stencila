// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to replace some block content with new block content.
 */
export class ReplaceBlock extends SuggestionBlock {
  type = "ReplaceBlock";

  /**
   * The new replacement block content.
   */
  replacement: Block[];

  constructor(content: Block[], replacement: Block[], options?: Partial<ReplaceBlock>) {
    super(content);
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
