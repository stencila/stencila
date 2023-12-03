// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { ModifyOperation } from "./ModifyOperation.js";
import { SuggestionInline } from "./SuggestionInline.js";

/**
 * A suggestion to modify some inline content.
 */
export class ModifyInline extends SuggestionInline {
  type = "ModifyInline";

  /**
   * The operations to be applied to the nodes.
   */
  operations: ModifyOperation[];

  constructor(content: Inline[], operations: ModifyOperation[], options?: Partial<ModifyInline>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
    this.operations = operations;
  }
}

/**
* Create a new `ModifyInline`
*/
export function modifyInline(content: Inline[], operations: ModifyOperation[], options?: Partial<ModifyInline>): ModifyInline {
  return new ModifyInline(content, operations, options);
}
