// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
import { ExecutionDependantNode } from "./ExecutionDependantNode.js";
import { ExecutionDependantRelation } from "./ExecutionDependantRelation.js";

/**
 * A downstream execution dependant of a node.
 */
export class ExecutionDependant extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionDependant";

  /**
   * The relation to the dependant.
   */
  dependantRelation: ExecutionDependantRelation;

  /**
   * The node that is the dependant.
   */
  dependantNode: ExecutionDependantNode;

  /**
   * The location that the dependant is defined.
   */
  codeLocation?: CodeLocation;

  constructor(dependantRelation: ExecutionDependantRelation, dependantNode: ExecutionDependantNode, options?: Partial<ExecutionDependant>) {
    super();
    this.type = "ExecutionDependant";
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
