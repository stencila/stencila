// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Executable } from "./Executable.js";
import { IfClause } from "./IfClause.js";

/**
 * Show and execute alternative content conditional upon an executed expression.
 */
export class If extends Executable {
  type = "If";

  /**
   * The clauses making up the `If` node
   */
  clauses: IfClause[];

  constructor(clauses: IfClause[], options?: Partial<If>) {
    super();
    if (options) Object.assign(this, options);
    this.clauses = clauses;
  }
}

/**
* Create a new `If`
*/
export function if_(clauses: IfClause[], options?: Partial<If>): If {
  return new If(clauses, options);
}
