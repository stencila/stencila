// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { ModifyOperation } from "./ModifyOperation.js";
import { SuggestionInline } from "./SuggestionInline.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * A suggestion to modify some inline content.
 */
export class ModifyInline extends SuggestionInline {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ModifyInline";

  /**
   * The operations to be applied to the nodes.
   */
  operations: ModifyOperation[];

  constructor(suggestionStatus: SuggestionStatus, content: Inline[], operations: ModifyOperation[], options?: Partial<ModifyInline>) {
    super(suggestionStatus, content);
    this.type = "ModifyInline";
    if (options) Object.assign(this, options);
    this.suggestionStatus = suggestionStatus;
    this.content = content;
    this.operations = operations;
  }
}

/**
* Create a new `ModifyInline`
*/
export function modifyInline(suggestionStatus: SuggestionStatus, content: Inline[], operations: ModifyOperation[], options?: Partial<ModifyInline>): ModifyInline {
  return new ModifyInline(suggestionStatus, content, operations, options);
}
