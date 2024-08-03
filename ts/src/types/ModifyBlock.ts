// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ModifyOperation } from "./ModifyOperation.js";
import { SuggestionBlock } from "./SuggestionBlock.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to modify some block content.
 */
export class ModifyBlock extends SuggestionBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ModifyBlock";

  /**
   * The operations to be applied to the nodes.
   */
  operations: ModifyOperation[];

  constructor(suggestionStatus: SuggestionStatus, content: Block[], operations: ModifyOperation[], options?: Partial<ModifyBlock>) {
    super(suggestionStatus, content);
    this.type = "ModifyBlock";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
    this.operations = operations;
  }
}

/**
* Create a new `ModifyBlock`
*/
export function modifyBlock(suggestionStatus: SuggestionStatus, content: Block[], operations: ModifyOperation[], options?: Partial<ModifyBlock>): ModifyBlock {
  return new ModifyBlock(suggestionStatus, content, operations, options);
}
