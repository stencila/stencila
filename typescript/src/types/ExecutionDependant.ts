// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { ExecutionDependantNode } from "./ExecutionDependantNode.js";
import { ExecutionDependantRelation } from "./ExecutionDependantRelation.js";
import { Integer } from "./Integer.js";

/**
 * A downstream execution dependant of a node
 */
export class ExecutionDependant extends Entity {
  type = "ExecutionDependant";

  /**
   * The relation to the dependant
   */
  dependantRelation: ExecutionDependantRelation;

  /**
   * The node that is the dependant
   */
  dependantNode: ExecutionDependantNode;

  /**
   * The location that the dependant is defined within code
   */
  codeLocation?: Integer[];

  constructor(dependantRelation: ExecutionDependantRelation, dependantNode: ExecutionDependantNode, options?: Partial<ExecutionDependant>) {
    super();
    if (options) Object.assign(this, options);
    this.dependantRelation = dependantRelation;
    this.dependantNode = dependantNode;
  }
}

/**
* Create a new `ExecutionDependant`
*/
export function executionDependant(dependantRelation: ExecutionDependantRelation, dependantNode: ExecutionDependantNode, options?: Partial<ExecutionDependant>): ExecutionDependant {
  return new ExecutionDependant(dependantRelation, dependantNode, options);
}
