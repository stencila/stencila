// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";

/**
 * A clause within an `IfBlock` node.
 */
export class IfBlockClause extends CodeExecutable {
  type = "IfBlockClause";

  /**
   * Whether this clause is the active clause in the parent `IfBlock` node
   */
  isActive?: boolean;

  /**
   * The content to render if the result is truthy
   */
  content: Block[];

  constructor(code: Cord, content: Block[], options?: Partial<IfBlockClause>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.content = content;
  }
}

/**
* Create a new `IfBlockClause`
*/
export function ifBlockClause(code: Cord, content: Block[], options?: Partial<IfBlockClause>): IfBlockClause {
  return new IfBlockClause(code, content, options);
}
