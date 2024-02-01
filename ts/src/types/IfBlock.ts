// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Executable } from "./Executable.js";
import { IfBlockClause } from "./IfBlockClause.js";

/**
 * Show and execute alternative content conditional upon an executed expression.
 */
export class IfBlock extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "IfBlock";

  /**
   * The clauses making up the `IfBlock` node
   */
  clauses: IfBlockClause[];

  /**
   * The authors of the if block.
   */
  authors?: Author[];

  constructor(clauses: IfBlockClause[], options?: Partial<IfBlock>) {
    super();
    this.type = "IfBlock";
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
