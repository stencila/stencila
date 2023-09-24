// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Executable } from './Executable';
import { IfClause } from './IfClause';

// Show and execute alternative content conditional upon an executed expression
export class If extends Executable {
  type = "If";

  // The clauses making up the `If` node
  clauses: IfClause[];

  constructor(clauses: IfClause[], options?: If) {
    super()
    if (options) Object.assign(this, options)
    this.clauses = clauses;
  }

  static from(other: If): If {
    return new If(other.clauses!, other)
  }
}
