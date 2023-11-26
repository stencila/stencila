// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { ModifyOperation } from "./ModifyOperation.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A suggestion to modify some block content.
 */
export class ModifyBlock extends SuggestionBlock {
  type = "ModifyBlock";

  constructor(content: Block[], operations: ModifyOperation[], options?: Partial<ModifyBlock>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
    this.operations = operations;
  }
}

/**
* Create a new `ModifyBlock`
*/
export function modifyBlock(content: Block[], operations: ModifyOperation[], options?: Partial<ModifyBlock>): ModifyBlock {
  return new ModifyBlock(content, operations, options);
}
