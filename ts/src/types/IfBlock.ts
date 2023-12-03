// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Executable } from "./Executable.js";
import { IfBlockClause } from "./IfBlockClause.js";

/**
 * Show and execute alternative content conditional upon an executed expression.
 */
export class IfBlock extends Executable {
  type = "IfBlock";

  /**
   * The clauses making up the `IfBlock` node
   */
  clauses: IfBlockClause[];

  constructor(clauses: IfBlockClause[], options?: Partial<IfBlock>) {
    super();
    if (options) Object.assign(this, options);
    this.clauses = clauses;
  }
}

/**
* Create a new `IfBlock`
*/
export function ifBlock(clauses: IfBlockClause[], options?: Partial<IfBlock>): IfBlock {
  return new IfBlock(clauses, options);
}
